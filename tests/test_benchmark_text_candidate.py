import importlib.util
import pathlib
import unittest

path = pathlib.Path(__file__).resolve().parents[1] / "scripts" / "benchmark_text_candidate.py"
spec = importlib.util.spec_from_file_location("candidate_benchmark", path)
bench = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bench)

def profile(backend="Metal"):
    return {"event":"generation_summary", "status":"complete", "contaminated":False,
            "counter_overflow":False,"dropped_measurements":0,"backend":backend,
            "compute_pass_count":1,"gpu_timestamp_enabled":True,
            "stages":[{"phase":"prefill","stage":"ffn_proj",
                       "totals":{"compute_passes":1,"calls":1,"gpu_measured_calls":1,"gpu_ns_sum":400}}]}

class BenchmarkContracts(unittest.TestCase):
    def test_vectorization_does_not_leak_into_projection_baseline(self):
        from types import SimpleNamespace
        args = SimpleNamespace(baseline_ffn=False, cooperative_down=True, cooperative_qkv=True, cooperative_attn_proj=True, baseline_projections=True, vectorized_ffn=True)
        baseline = bench.experiment_flags(False,args)
        candidate = bench.experiment_flags(True,args)
        self.assertEqual(baseline["AIRFRAME_EXPERIMENTAL_VECTORIZED_FFN"],"0")
        self.assertEqual(list(baseline.values()).count("1"),4)
        self.assertEqual(set(candidate.values()),{"1"})

    def test_candidate_flags_are_explicit_and_isolated(self):
        from types import SimpleNamespace
        args = SimpleNamespace(baseline_ffn=False, cooperative_down=True, cooperative_qkv=True, cooperative_attn_proj=True, baseline_projections=False, vectorized_ffn=True)
        self.assertEqual(set(bench.experiment_flags(False, args).values()), {"0"})
        self.assertEqual(set(bench.experiment_flags(True, args).values()), {"1"})
        args.baseline_ffn = True
        flags = bench.experiment_flags(False, args)
        self.assertEqual(flags["AIRFRAME_EXPERIMENTAL_STAGED_FFN"], "1")
        self.assertEqual(flags["AIRFRAME_EXPERIMENTAL_COOPERATIVE_FFN_DOWN"], "1")
        self.assertEqual(flags["AIRFRAME_EXPERIMENTAL_COOPERATIVE_QKV"], "0")
        self.assertEqual(flags["AIRFRAME_EXPERIMENTAL_COOPERATIVE_ATTN_PROJ"], "0")
        args = SimpleNamespace(baseline_ffn=False, cooperative_down=False, cooperative_qkv=False, cooperative_attn_proj=False, baseline_projections=False, vectorized_ffn=False)
        flags = bench.experiment_flags(True, args)
        self.assertEqual(flags["AIRFRAME_EXPERIMENTAL_STAGED_FFN"], "1")
        self.assertEqual(list(flags.values()).count("0"), 4)

    def test_cpu_or_vulkan_is_not_a_metal_pass(self):
        self.assertEqual(len(bench.validate_profiles([profile()],1,True)),1)
        with self.assertRaises(ValueError):
            bench.validate_profiles([profile("Vulkan")],1,True)

    def test_partial_capture_and_missing_terminal_fail(self):
        for field in ("contaminated","counter_overflow","dropped_measurements"):
            row=profile();row[field]=1
            with self.assertRaises(ValueError):
                bench.validate_profiles([row],1)
        row=profile();row["stages"][0]["totals"]["gpu_measured_calls"]=0
        with self.assertRaises(ValueError):
            bench.validate_profiles([row],1)
        with self.assertRaises(ValueError):
            bench.validate_profiles([],1)

    def test_same_text_different_usage_or_reason_is_not_parity(self):
        row={"text":"Fastembed","usage":{"prompt_tokens":82,"completion_tokens":3},"finish_reason":"stop"}
        for field,value in (("text","wrong"),("usage",{"prompt_tokens":82,"completion_tokens":4}),("finish_reason","length")):
            changed=dict(row);changed[field]=value
            with self.assertRaises(ValueError):
                bench.compare_outputs([row],[changed])
        bench.compare_outputs([row],[dict(row)])
        with self.assertRaises(ValueError):
            bench.compare_outputs([row],[])

    def test_missing_gpu_duration_stays_unavailable(self):
        row=profile();row["stages"][0]["totals"].update(gpu_measured_calls=0,gpu_ns_sum=None)
        self.assertIsNone(bench.stage_totals([row])["prefill.ffn_proj"]["gpu_ns"])

if __name__=="__main__":
    unittest.main()
