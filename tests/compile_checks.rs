/// Compile-time assertions: template files referenced by include_str! exist.

#[test]
fn test_template_files_exist() {
    let docker = include_str!("../templates/docker/Dockerfile");
    assert!(!docker.is_empty());
    assert!(docker.contains("FROM"));

    let compose = include_str!("../templates/docker/docker-compose.yml");
    assert!(!compose.is_empty());
    assert!(compose.contains("services:"));

    let nginx = include_str!("../templates/docker/nginx.conf");
    assert!(!nginx.is_empty());

    let k8s_deploy = include_str!("../templates/kubernetes/deployment.yaml");
    assert!(!k8s_deploy.is_empty());
    assert!(k8s_deploy.contains("apiVersion:"));

    let k8s_svc = include_str!("../templates/kubernetes/service.yaml");
    assert!(!k8s_svc.is_empty());

    let k8s_cm = include_str!("../templates/kubernetes/configmap.yaml");
    assert!(!k8s_cm.is_empty());

    let railway = include_str!("../templates/railway/railway.toml");
    assert!(!railway.is_empty());

    let fly = include_str!("../templates/fly/fly.toml");
    assert!(!fly.is_empty());

    let fastapi_main = include_str!("../templates/frameworks/fastapi/main.py");
    assert!(!fastapi_main.is_empty());
    assert!(fastapi_main.contains("FastAPI"));

    let fastapi_reqs = include_str!("../templates/frameworks/fastapi/requirements.txt");
    assert!(!fastapi_reqs.is_empty());

    let express_app = include_str!("../templates/frameworks/express/app.js");
    assert!(!express_app.is_empty());
    assert!(express_app.contains("express"));

    let express_pkg = include_str!("../templates/frameworks/express/package.json");
    assert!(!express_pkg.is_empty());
}
