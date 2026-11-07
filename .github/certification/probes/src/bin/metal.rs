use serde_json::{json, Value};
use std::{sync::mpsc, time::Duration};
use wgpu::util::DeviceExt;

async fn diagnose(vulkan: bool) -> Value {
    let backend = if vulkan {
        wgpu::Backends::VULKAN
    } else {
        wgpu::Backends::METAL
    };
    let requested = if vulkan { "Vulkan" } else { "Metal" };
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: backend,
        ..Default::default()
    });
    let adapters = instance.enumerate_adapters(backend);
    let facts: Vec<Value> = adapters.iter().map(|a| {
        let i = a.get_info(); let l = a.limits();
        json!({"name":i.name,"backend":format!("{:?}",i.backend),"device_type":format!("{:?}",i.device_type),
               "features":format!("{:?}",a.features()),"limits":format!("{:?}",l),
               "max_storage_buffer_binding_size":l.max_storage_buffer_binding_size,
               "max_buffer_size":l.max_buffer_size,"max_storage_buffers_per_shader_stage":l.max_storage_buffers_per_shader_stage,
               "max_compute_invocations_per_workgroup":l.max_compute_invocations_per_workgroup})
    }).collect();
    if adapters.is_empty() {
        return json!({"requested_backend":requested,"status":"METAL_UNAVAILABLE","adapters":facts,"compute":"NOT_RUN","reason":"No adapter enumerated for the explicitly selected backend"});
    }
    let adapter = adapters
        .iter()
        .find(|a| a.get_info().device_type == wgpu::DeviceType::DiscreteGpu)
        .or_else(|| {
            adapters
                .iter()
                .find(|a| a.get_info().device_type == wgpu::DeviceType::IntegratedGpu)
        })
        .unwrap_or(&adapters[0]);
    let selected_index = adapters
        .iter()
        .position(|a| a.get_info() == adapter.get_info())
        .unwrap();
    let cap = adapter.limits();
    let mut limits = wgpu::Limits::downlevel_defaults();
    limits.max_storage_buffer_binding_size = cap.max_storage_buffer_binding_size;
    limits.max_buffer_size = cap.max_buffer_size;
    limits.max_storage_buffers_per_shader_stage = cap.max_storage_buffers_per_shader_stage.max(14);
    limits.max_compute_invocations_per_workgroup = 256;
    let required = format!("{:?}", limits);
    let (device, queue) = match adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::Performance,
            ..Default::default()
        })
        .await
    {
        Ok(value) => value,
        Err(e) => {
            return json!({"requested_backend":requested,"status":"REQUIRED_DEVICE_UNAVAILABLE","adapters":facts,"selected":facts[selected_index],"required_limits":required,"error":e.to_string(),"compute":"NOT_RUN"})
        }
    };
    let storage = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("diagnostic arithmetic"),
        contents: &[0u8; 128],
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("diagnostic readback"),
        size: 128,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label:Some("diagnostic arithmetic"),source:wgpu::ShaderSource::Wgsl(
            "@group(0) @binding(0) var<storage, read_write> values: array<f32>;\n@compute @workgroup_size(256) fn main(@builtin(global_invocation_id) id:vec3<u32>){ if(id.x<32u){values[id.x]=f32(id.x)*2.0+1.0;} }".into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("diagnostic arithmetic"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("diagnostic arithmetic"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage.as_entire_binding(),
        }],
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("diagnostic arithmetic"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&storage, 0, &readback, 0, 128);
    let submission = queue.submit(Some(encoder.finish()));
    let (tx, rx) = mpsc::channel();
    readback.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    let poll = device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission),
        timeout: Some(Duration::from_secs(60)),
    });
    if let Err(e) = poll {
        return json!({"requested_backend":requested,"status":"COMPUTE_FAILED","adapters":facts,"selected":facts[selected_index],"error":e.to_string(),"compute":"FAIL"});
    }
    rx.recv_timeout(Duration::from_secs(60))
        .expect("readback callback timeout")
        .expect("readback failed");
    let range = readback.slice(..).get_mapped_range();
    let actual: Vec<f32> = range
        .as_chunks::<4>()
        .0
        .iter()
        .map(|v| f32::from_le_bytes(*v))
        .collect();
    let expected: Vec<f32> = (0..32).map(|i| (i * 2 + 1) as f32).collect();
    let passed = actual == expected;
    json!({"requested_backend":requested,"status":if passed{"COMPUTE_PASS"}else{"COMPUTE_FAILED"},"adapters":facts,
           "selected":facts[selected_index],"required_limits":required,"compute":if passed{"PASS"}else{"FAIL"},
           "actual":actual,"expected":expected,"numerical_model_certification":"NOT_RUN"})
}
fn main() {
    let vulkan = std::env::args().any(|v| v == "--vulkan");
    println!("{}", pollster::block_on(diagnose(vulkan)));
}
