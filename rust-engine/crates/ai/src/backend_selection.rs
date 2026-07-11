//! Capability reporting and conservative inference backend selection.

use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const INFERENCE_BACKEND_PLAN_SCHEMA: &str = "monogatari-inference-backend-plan/v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InferenceBackendId {
    WebGpu,
    LlamaCpp,
    WinMlGenAi,
    DirectMlOnnx,
    MlxLm,
    Vllm,
    Sglang,
    OpenAiCompatible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendReadiness {
    Ready,
    ProbeRequired,
    SetupRequired,
    Blocked,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentTarget {
    Web,
    Desktop,
    Server,
}

impl Default for DeploymentTarget {
    fn default() -> Self {
        Self::Desktop
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProfile {
    Qwen35Text08B,
    GenericCausalLm,
}

impl Default for ModelProfile {
    fn default() -> Self {
        Self::Qwen35Text08B
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccelerationKind {
    Cpu,
    Cuda,
    Vulkan,
    Metal,
    Rocm,
    Musa,
    Sycl,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RuntimeProbeSignals {
    /// The client has a WebGPU adapter, but has not necessarily loaded the model.
    pub webgpu_adapter_available: bool,
    /// The exact WebGPU model profile completed initialization and generation.
    pub webgpu_model_ready: bool,
    /// A llama.cpp OpenAI-compatible endpoint completed health and generation probes.
    pub llama_cpp_service_ready: bool,
    /// A GGUF artifact matching the selected model profile is present.
    pub gguf_model_available: bool,
    /// The selected Hugging Face model artifacts are present for a server runtime.
    pub hf_model_available: bool,
    /// A WinML + ORT GenAI model-level generation probe completed successfully.
    pub winml_qwen35_ready: bool,
    /// A compatible full-sequence ONNX model completed a DirectML generation probe.
    pub directml_model_ready: bool,
    /// An MLX-LM model-level generation probe completed successfully.
    pub mlx_model_ready: bool,
    /// A vLLM OpenAI-compatible endpoint completed health and generation probes.
    pub vllm_service_ready: bool,
    /// An SGLang OpenAI-compatible endpoint completed health and generation probes.
    pub sglang_service_ready: bool,
    /// A user-configured OpenAI-compatible API passed local configuration validation.
    pub api_configured: bool,
    /// The configured OpenAI-compatible API completed a model-level generation probe.
    pub api_service_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BackendPlanRequest {
    pub target: DeploymentTarget,
    pub model_profile: ModelProfile,
    pub signals: RuntimeProbeSignals,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostCapabilities {
    pub os: String,
    pub architecture: String,
    pub is_wsl: bool,
    pub cuda_runtime_detected: bool,
    pub vulkan_probe_detected: bool,
    pub metal_runtime_detected: bool,
    pub rocm_runtime_detected: bool,
    pub musa_runtime_detected: bool,
    pub sycl_runtime_detected: bool,
    pub winml_runtime_detected: bool,
    pub directml_runtime_detected: bool,
    pub llama_cpp_detected: bool,
    pub mlx_lm_detected: bool,
    pub vllm_detected: bool,
    pub sglang_detected: bool,
    pub docker_detected: bool,
}

impl Default for HostCapabilities {
    fn default() -> Self {
        Self {
            os: env::consts::OS.to_string(),
            architecture: env::consts::ARCH.to_string(),
            is_wsl: false,
            cuda_runtime_detected: false,
            vulkan_probe_detected: false,
            metal_runtime_detected: false,
            rocm_runtime_detected: false,
            musa_runtime_detected: false,
            sycl_runtime_detected: false,
            winml_runtime_detected: false,
            directml_runtime_detected: false,
            llama_cpp_detected: false,
            mlx_lm_detected: false,
            vllm_detected: false,
            sglang_detected: false,
            docker_detected: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendAssessment {
    pub backend: InferenceBackendId,
    pub readiness: BackendReadiness,
    pub reason_code: String,
    pub summary: String,
    pub accelerators: Vec<AccelerationKind>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InferenceBackendPlan {
    pub schema: String,
    pub target: DeploymentTarget,
    pub model_profile: ModelProfile,
    pub host: HostCapabilities,
    /// A backend is recommended only after a model-level probe succeeds.
    pub recommended_backend: Option<InferenceBackendId>,
    /// First detected backend that still needs a model-level probe.
    pub next_probe: Option<InferenceBackendId>,
    pub fallback_order: Vec<InferenceBackendId>,
    pub selection_summary: String,
    pub backends: Vec<BackendAssessment>,
}

pub fn detect_host_capabilities() -> HostCapabilities {
    let os = env::consts::OS.to_string();
    let is_windows = os == "windows";
    let is_linux = os == "linux";
    let is_macos = os == "macos";
    let is_wsl = is_linux
        && std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .map(|release| release.to_ascii_lowercase().contains("microsoft"))
            .unwrap_or(false);

    HostCapabilities {
        os,
        architecture: env::consts::ARCH.to_string(),
        is_wsl,
        cuda_runtime_detected: command_exists_any(&["nvidia-smi"])
            || Path::new("/usr/lib/wsl/lib/nvidia-smi").is_file()
            || Path::new("/dev/nvidiactl").exists(),
        // A loader library alone is insufficient evidence. vulkaninfo is the
        // minimum host-side probe and the model still needs an end-to-end run.
        vulkan_probe_detected: command_exists_any(&["vulkaninfo"]),
        metal_runtime_detected: is_macos,
        rocm_runtime_detected: command_exists_any(&["rocminfo", "rocm-smi"])
            || Path::new("/dev/kfd").exists(),
        musa_runtime_detected: command_exists_any(&["mthreads-gmi", "musa-smi"])
            || env_path_exists("MUSA_HOME"),
        sycl_runtime_detected: command_exists_any(&["sycl-ls"]) || env_path_exists("ONEAPI_ROOT"),
        winml_runtime_detected: is_windows
            && windows_system_file_exists("Microsoft.Windows.AI.MachineLearning.dll"),
        directml_runtime_detected: is_windows
            && (windows_system_file_exists("DirectML.dll")
                || windows_system_file_exists("d3d12.dll")),
        llama_cpp_detected: command_exists_any(&["llama-server", "llama-server.exe"])
            || env_file_exists("MONOGATARI_LLAMA_SERVER"),
        mlx_lm_detected: command_exists_any(&["mlx_lm.generate", "mlx_lm.server"]),
        vllm_detected: command_exists_any(&["vllm"]),
        sglang_detected: command_exists_any(&["sglang"]),
        docker_detected: command_exists_any(&["docker", "podman"]),
    }
}

pub fn build_inference_backend_plan(
    host: HostCapabilities,
    request: BackendPlanRequest,
) -> InferenceBackendPlan {
    let accelerators = detected_accelerators(&host);
    let mut backends = vec![
        assess_webgpu(&request),
        assess_llama_cpp(&host, &request, &accelerators),
        assess_winml(&host, &request),
        assess_directml(&host, &request),
        assess_mlx(&host, &request),
        assess_vllm(&host, &request),
        assess_sglang(&host, &request),
        assess_api(&request),
    ];
    let preferred = preferred_order(request.target, &host);
    backends.sort_by_key(|assessment| {
        preferred
            .iter()
            .position(|backend| *backend == assessment.backend)
            .unwrap_or(usize::MAX)
    });

    let recommended_backend = preferred
        .iter()
        .copied()
        .find(|backend| readiness_for(&backends, *backend) == Some(BackendReadiness::Ready));
    let next_probe = preferred.iter().copied().find(|backend| {
        readiness_for(&backends, *backend) == Some(BackendReadiness::ProbeRequired)
    });
    let fallback_order = preferred
        .into_iter()
        .filter(|backend| {
            matches!(
                readiness_for(&backends, *backend),
                Some(
                    BackendReadiness::Ready
                        | BackendReadiness::ProbeRequired
                        | BackendReadiness::SetupRequired
                )
            )
        })
        .collect::<Vec<_>>();
    let selection_summary = recommended_backend
        .map(|backend| {
            format!(
                "Selected {} because its current model-level probe is ready.",
                backend_label(backend)
            )
        })
        .unwrap_or_else(|| {
            "No backend is recommended until a model-level generation probe succeeds.".to_string()
        });

    InferenceBackendPlan {
        schema: INFERENCE_BACKEND_PLAN_SCHEMA.to_string(),
        target: request.target,
        model_profile: request.model_profile,
        host,
        recommended_backend,
        next_probe,
        fallback_order,
        selection_summary,
        backends,
    }
}

fn assess_webgpu(request: &BackendPlanRequest) -> BackendAssessment {
    if request.target == DeploymentTarget::Server {
        return assessment(
            InferenceBackendId::WebGpu,
            BackendReadiness::Unavailable,
            "client_runtime_only",
            "WebGPU is a browser or WebView client runtime, not a server backend.",
            vec![],
        );
    }
    if request.signals.webgpu_model_ready {
        return assessment(
            InferenceBackendId::WebGpu,
            BackendReadiness::Ready,
            "model_probe_passed",
            "The packaged WebGPU model completed initialization and generation.",
            vec![],
        );
    }
    if request.signals.webgpu_adapter_available {
        return assessment(
            InferenceBackendId::WebGpu,
            BackendReadiness::ProbeRequired,
            "adapter_detected_model_unverified",
            "A WebGPU adapter is available; initialize and generate with the exact model profile before selection.",
            vec![],
        );
    }
    assessment(
        InferenceBackendId::WebGpu,
        BackendReadiness::Unavailable,
        "adapter_not_detected",
        "No client WebGPU adapter was reported.",
        vec![],
    )
}

fn assess_llama_cpp(
    host: &HostCapabilities,
    request: &BackendPlanRequest,
    accelerators: &[AccelerationKind],
) -> BackendAssessment {
    if request.signals.llama_cpp_service_ready {
        return assessment(
            InferenceBackendId::LlamaCpp,
            BackendReadiness::Ready,
            "openai_service_probe_passed",
            "The llama.cpp health, completion, and streaming probes passed.",
            accelerators.to_vec(),
        );
    }
    if host.llama_cpp_detected && request.signals.gguf_model_available {
        return assessment(
            InferenceBackendId::LlamaCpp,
            BackendReadiness::ProbeRequired,
            "runtime_and_model_detected",
            "llama.cpp and a matching GGUF are present; run device listing, generation, and SSE probes.",
            accelerators.to_vec(),
        );
    }
    if host.llama_cpp_detected {
        return assessment(
            InferenceBackendId::LlamaCpp,
            BackendReadiness::SetupRequired,
            "gguf_model_missing",
            "llama.cpp is present but no matching GGUF artifact was reported.",
            accelerators.to_vec(),
        );
    }
    assessment(
        InferenceBackendId::LlamaCpp,
        BackendReadiness::SetupRequired,
        "runtime_not_installed",
        "Install a platform-specific llama.cpp runtime and matching GGUF before probing.",
        accelerators.to_vec(),
    )
}

fn assess_winml(host: &HostCapabilities, request: &BackendPlanRequest) -> BackendAssessment {
    if host.os != "windows" {
        return assessment(
            InferenceBackendId::WinMlGenAi,
            BackendReadiness::Unavailable,
            "windows_only",
            "Windows ML is available only on Windows.",
            vec![],
        );
    }
    if request.signals.winml_qwen35_ready {
        return assessment(
            InferenceBackendId::WinMlGenAi,
            BackendReadiness::Ready,
            "model_probe_passed",
            "The WinML + ORT GenAI model-level generation probe passed.",
            vec![],
        );
    }
    if request.model_profile == ModelProfile::Qwen35Text08B {
        return assessment(
            InferenceBackendId::WinMlGenAi,
            BackendReadiness::Blocked,
            "qwen35_dml_graph_capture_partition_failure",
            "Qwen3.5 is blocked until all required hybrid operators can execute under the selected WinML provider without an invalid DML graph capture.",
            vec![],
        );
    }
    if host.winml_runtime_detected {
        return assessment(
            InferenceBackendId::WinMlGenAi,
            BackendReadiness::ProbeRequired,
            "runtime_detected_model_unverified",
            "WinML is present; the exact exported model still requires a generation probe.",
            vec![],
        );
    }
    assessment(
        InferenceBackendId::WinMlGenAi,
        BackendReadiness::SetupRequired,
        "runtime_not_detected",
        "A compatible Windows ML runtime was not detected.",
        vec![],
    )
}

fn assess_directml(host: &HostCapabilities, request: &BackendPlanRequest) -> BackendAssessment {
    if host.os != "windows" {
        return assessment(
            InferenceBackendId::DirectMlOnnx,
            BackendReadiness::Unavailable,
            "windows_only",
            "The linked DirectML executor is available only in Windows builds.",
            vec![],
        );
    }
    if request.signals.directml_model_ready {
        return assessment(
            InferenceBackendId::DirectMlOnnx,
            BackendReadiness::Ready,
            "model_probe_passed",
            "A compatible full-sequence ONNX model completed DirectML generation.",
            vec![],
        );
    }
    if request.model_profile == ModelProfile::Qwen35Text08B {
        return assessment(
            InferenceBackendId::DirectMlOnnx,
            BackendReadiness::Blocked,
            "qwen35_hybrid_contract_unsupported",
            "The current executor cannot drive Qwen3.5 hybrid recurrent state inputs, and older DirectML runtimes lack required operators.",
            vec![],
        );
    }
    if host.directml_runtime_detected {
        return assessment(
            InferenceBackendId::DirectMlOnnx,
            BackendReadiness::ProbeRequired,
            "runtime_detected_model_unverified",
            "DirectML is present; a compatible full-sequence causal-LM graph must pass generation.",
            vec![],
        );
    }
    assessment(
        InferenceBackendId::DirectMlOnnx,
        BackendReadiness::Unavailable,
        "runtime_not_detected",
        "DirectML runtime support was not detected.",
        vec![],
    )
}

fn assess_mlx(host: &HostCapabilities, request: &BackendPlanRequest) -> BackendAssessment {
    if host.os != "macos" {
        return assessment(
            InferenceBackendId::MlxLm,
            BackendReadiness::Unavailable,
            "macos_only",
            "MLX-LM is a macOS Apple Silicon backend.",
            vec![AccelerationKind::Metal],
        );
    }
    if request.signals.mlx_model_ready {
        return assessment(
            InferenceBackendId::MlxLm,
            BackendReadiness::Ready,
            "model_probe_passed",
            "The MLX-LM model-level generation probe passed.",
            vec![AccelerationKind::Metal],
        );
    }
    if host.mlx_lm_detected {
        return assessment(
            InferenceBackendId::MlxLm,
            BackendReadiness::ProbeRequired,
            "runtime_detected_model_unverified",
            "MLX-LM is present; convert and generate with the exact model profile.",
            vec![AccelerationKind::Metal],
        );
    }
    assessment(
        InferenceBackendId::MlxLm,
        BackendReadiness::SetupRequired,
        "runtime_not_installed",
        "Install and pin MLX-LM before running the model probe.",
        vec![AccelerationKind::Metal],
    )
}

fn assess_vllm(host: &HostCapabilities, request: &BackendPlanRequest) -> BackendAssessment {
    if request.target != DeploymentTarget::Server {
        return assessment(
            InferenceBackendId::Vllm,
            BackendReadiness::Unavailable,
            "server_profile_only",
            "vLLM is treated as a managed Linux service, not a bundled client runtime.",
            vec![],
        );
    }
    if request.signals.vllm_service_ready {
        return assessment(
            InferenceBackendId::Vllm,
            BackendReadiness::Ready,
            "openai_service_probe_passed",
            "The vLLM health and OpenAI-compatible generation probes passed.",
            detected_accelerators(host),
        );
    }
    if host.vllm_detected && request.signals.hf_model_available {
        return assessment(
            InferenceBackendId::Vllm,
            BackendReadiness::ProbeRequired,
            "runtime_and_model_detected",
            "vLLM and model artifacts are present; run architecture, generation, and concurrency probes.",
            detected_accelerators(host),
        );
    }
    assessment(
        InferenceBackendId::Vllm,
        BackendReadiness::SetupRequired,
        "service_profile_not_ready",
        "Install a pinned vLLM service image or environment and stage the model artifacts.",
        detected_accelerators(host),
    )
}

fn assess_sglang(host: &HostCapabilities, request: &BackendPlanRequest) -> BackendAssessment {
    if request.target != DeploymentTarget::Server {
        return assessment(
            InferenceBackendId::Sglang,
            BackendReadiness::Unavailable,
            "server_profile_only",
            "SGLang is treated as a managed Linux service, not a bundled client runtime.",
            vec![],
        );
    }
    if request.signals.sglang_service_ready {
        return assessment(
            InferenceBackendId::Sglang,
            BackendReadiness::Ready,
            "openai_service_probe_passed",
            "The SGLang health and OpenAI-compatible generation probes passed.",
            detected_accelerators(host),
        );
    }
    if host.sglang_detected && request.signals.hf_model_available {
        return assessment(
            InferenceBackendId::Sglang,
            BackendReadiness::ProbeRequired,
            "runtime_and_model_detected",
            "SGLang and model artifacts are present; run backend-specific generation probes.",
            detected_accelerators(host),
        );
    }
    assessment(
        InferenceBackendId::Sglang,
        BackendReadiness::SetupRequired,
        "service_profile_not_ready",
        "Install a pinned SGLang service image or environment and stage the model artifacts.",
        detected_accelerators(host),
    )
}

fn assess_api(request: &BackendPlanRequest) -> BackendAssessment {
    if request.signals.api_service_ready {
        return assessment(
            InferenceBackendId::OpenAiCompatible,
            BackendReadiness::Ready,
            "generation_probe_passed",
            "The configured OpenAI-compatible endpoint passed a model-level generation probe.",
            vec![],
        );
    }
    if request.signals.api_configured {
        return assessment(
            InferenceBackendId::OpenAiCompatible,
            BackendReadiness::ProbeRequired,
            "configuration_validated_model_unverified",
            "The endpoint configuration is valid; run a model-level generation probe before selection.",
            vec![],
        );
    }
    assessment(
        InferenceBackendId::OpenAiCompatible,
        BackendReadiness::SetupRequired,
        "configuration_missing",
        "Configure an HTTPS endpoint, or an HTTP loopback endpoint for a local service.",
        vec![],
    )
}

fn preferred_order(target: DeploymentTarget, host: &HostCapabilities) -> Vec<InferenceBackendId> {
    match target {
        DeploymentTarget::Web => vec![
            InferenceBackendId::WebGpu,
            InferenceBackendId::OpenAiCompatible,
        ],
        DeploymentTarget::Server => vec![
            InferenceBackendId::Vllm,
            InferenceBackendId::Sglang,
            InferenceBackendId::LlamaCpp,
            InferenceBackendId::OpenAiCompatible,
        ],
        DeploymentTarget::Desktop if host.os == "macos" => vec![
            InferenceBackendId::LlamaCpp,
            InferenceBackendId::MlxLm,
            InferenceBackendId::WebGpu,
            InferenceBackendId::OpenAiCompatible,
        ],
        DeploymentTarget::Desktop if host.os == "windows" => vec![
            InferenceBackendId::WebGpu,
            InferenceBackendId::WinMlGenAi,
            InferenceBackendId::LlamaCpp,
            InferenceBackendId::DirectMlOnnx,
            InferenceBackendId::OpenAiCompatible,
        ],
        DeploymentTarget::Desktop => vec![
            InferenceBackendId::LlamaCpp,
            InferenceBackendId::WebGpu,
            InferenceBackendId::OpenAiCompatible,
        ],
    }
}

fn readiness_for(
    assessments: &[BackendAssessment],
    backend: InferenceBackendId,
) -> Option<BackendReadiness> {
    assessments
        .iter()
        .find(|assessment| assessment.backend == backend)
        .map(|assessment| assessment.readiness)
}

fn assessment(
    backend: InferenceBackendId,
    readiness: BackendReadiness,
    reason_code: &str,
    summary: &str,
    accelerators: Vec<AccelerationKind>,
) -> BackendAssessment {
    BackendAssessment {
        backend,
        readiness,
        reason_code: reason_code.to_string(),
        summary: summary.to_string(),
        accelerators,
    }
}

fn detected_accelerators(host: &HostCapabilities) -> Vec<AccelerationKind> {
    let mut accelerators = Vec::new();
    if host.cuda_runtime_detected {
        accelerators.push(AccelerationKind::Cuda);
    }
    if host.rocm_runtime_detected {
        accelerators.push(AccelerationKind::Rocm);
    }
    if host.musa_runtime_detected {
        accelerators.push(AccelerationKind::Musa);
    }
    if host.metal_runtime_detected {
        accelerators.push(AccelerationKind::Metal);
    }
    if host.vulkan_probe_detected {
        accelerators.push(AccelerationKind::Vulkan);
    }
    if host.sycl_runtime_detected {
        accelerators.push(AccelerationKind::Sycl);
    }
    accelerators.push(AccelerationKind::Cpu);
    accelerators
}

fn backend_label(backend: InferenceBackendId) -> &'static str {
    match backend {
        InferenceBackendId::WebGpu => "WebGPU",
        InferenceBackendId::LlamaCpp => "llama.cpp",
        InferenceBackendId::WinMlGenAi => "WinML GenAI",
        InferenceBackendId::DirectMlOnnx => "DirectML ONNX",
        InferenceBackendId::MlxLm => "MLX-LM",
        InferenceBackendId::Vllm => "vLLM",
        InferenceBackendId::Sglang => "SGLang",
        InferenceBackendId::OpenAiCompatible => "OpenAI-compatible API",
    }
}

fn command_exists_any(commands: &[&str]) -> bool {
    commands.iter().any(|command| command_exists(command))
}

fn command_exists(command: &str) -> bool {
    let command_path = Path::new(command);
    if command_path.components().count() > 1 {
        return command_path.is_file();
    }

    let Some(path) = env::var_os("PATH") else {
        return false;
    };
    let extensions = executable_extensions();
    env::split_paths(&path).any(|directory| {
        extensions.iter().any(|extension| {
            let mut candidate = directory.join(command);
            if !extension.is_empty() && candidate.extension().is_none() {
                candidate.set_extension(extension.trim_start_matches('.'));
            }
            candidate.is_file()
        })
    })
}

fn executable_extensions() -> Vec<String> {
    if cfg!(windows) {
        env::var("PATHEXT")
            .unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".to_string())
            .split(';')
            .map(|extension| extension.to_ascii_lowercase())
            .chain(std::iter::once(String::new()))
            .collect()
    } else {
        vec![String::new()]
    }
}

fn env_path_exists(name: &str) -> bool {
    env::var_os(name)
        .map(PathBuf::from)
        .is_some_and(|path| path.exists())
}

fn env_file_exists(name: &str) -> bool {
    env::var_os(name)
        .map(PathBuf::from)
        .is_some_and(|path| path.is_file())
}

fn windows_system_file_exists(filename: &str) -> bool {
    env::var_os("SystemRoot")
        .map(PathBuf::from)
        .map(|root| root.join("System32").join(filename))
        .is_some_and(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(os: &str) -> HostCapabilities {
        HostCapabilities {
            os: os.to_string(),
            architecture: "x86_64".to_string(),
            ..Default::default()
        }
    }

    fn request(target: DeploymentTarget) -> BackendPlanRequest {
        BackendPlanRequest {
            target,
            ..Default::default()
        }
    }

    fn backend<'a>(
        plan: &'a InferenceBackendPlan,
        id: InferenceBackendId,
    ) -> &'a BackendAssessment {
        plan.backends
            .iter()
            .find(|assessment| assessment.backend == id)
            .unwrap()
    }

    #[test]
    fn adapter_detection_does_not_claim_webgpu_readiness() {
        let mut request = request(DeploymentTarget::Web);
        request.signals.webgpu_adapter_available = true;

        let plan = build_inference_backend_plan(host("linux"), request);

        assert_eq!(plan.recommended_backend, None);
        assert_eq!(plan.next_probe, Some(InferenceBackendId::WebGpu));
        assert_eq!(
            backend(&plan, InferenceBackendId::WebGpu).readiness,
            BackendReadiness::ProbeRequired
        );
    }

    #[test]
    fn model_level_webgpu_probe_becomes_recommended() {
        let mut request = request(DeploymentTarget::Web);
        request.signals.webgpu_adapter_available = true;
        request.signals.webgpu_model_ready = true;
        request.signals.api_service_ready = true;

        let plan = build_inference_backend_plan(host("windows"), request);

        assert_eq!(plan.recommended_backend, Some(InferenceBackendId::WebGpu));
    }

    #[test]
    fn qwen35_native_windows_paths_remain_blocked_without_exact_probe() {
        let mut windows = host("windows");
        windows.winml_runtime_detected = true;
        windows.directml_runtime_detected = true;

        let plan = build_inference_backend_plan(windows, request(DeploymentTarget::Desktop));

        assert_eq!(
            backend(&plan, InferenceBackendId::WinMlGenAi).readiness,
            BackendReadiness::Blocked
        );
        assert_eq!(
            backend(&plan, InferenceBackendId::DirectMlOnnx).readiness,
            BackendReadiness::Blocked
        );
        assert!(!plan
            .fallback_order
            .contains(&InferenceBackendId::WinMlGenAi));
        assert!(!plan
            .fallback_order
            .contains(&InferenceBackendId::DirectMlOnnx));
    }

    #[test]
    fn explicit_winml_model_probe_can_replace_the_known_blocker() {
        let mut request = request(DeploymentTarget::Desktop);
        request.signals.winml_qwen35_ready = true;
        let mut windows = host("windows");
        windows.winml_runtime_detected = true;

        let plan = build_inference_backend_plan(windows, request);

        assert_eq!(
            plan.recommended_backend,
            Some(InferenceBackendId::WinMlGenAi)
        );
    }

    #[test]
    fn ready_llama_service_beats_remote_api_on_linux_desktop() {
        let mut request = request(DeploymentTarget::Desktop);
        request.signals.llama_cpp_service_ready = true;
        request.signals.api_service_ready = true;

        let plan = build_inference_backend_plan(host("linux"), request);

        assert_eq!(plan.recommended_backend, Some(InferenceBackendId::LlamaCpp));
    }

    #[test]
    fn server_prefers_ready_vllm_then_sglang_then_llama_cpp() {
        let mut request = request(DeploymentTarget::Server);
        request.signals.vllm_service_ready = true;
        request.signals.sglang_service_ready = true;
        request.signals.llama_cpp_service_ready = true;

        let plan = build_inference_backend_plan(host("linux"), request);

        assert_eq!(plan.recommended_backend, Some(InferenceBackendId::Vllm));
        assert_eq!(
            &plan.fallback_order[..3],
            &[
                InferenceBackendId::Vllm,
                InferenceBackendId::Sglang,
                InferenceBackendId::LlamaCpp,
            ]
        );
    }

    #[test]
    fn report_uses_versioned_schema() {
        let plan = build_inference_backend_plan(host("linux"), request(DeploymentTarget::Desktop));

        assert_eq!(plan.schema, INFERENCE_BACKEND_PLAN_SCHEMA);
        assert!(plan.selection_summary.contains("No backend"));
    }

    #[test]
    fn backend_ids_use_stable_snake_case_wire_names() {
        assert_eq!(
            serde_json::to_string(&InferenceBackendId::WebGpu).unwrap(),
            "\"web_gpu\""
        );
        assert_eq!(
            serde_json::to_string(&InferenceBackendId::WinMlGenAi).unwrap(),
            "\"win_ml_gen_ai\""
        );
        assert_eq!(
            serde_json::to_string(&InferenceBackendId::OpenAiCompatible).unwrap(),
            "\"open_ai_compatible\""
        );
        assert_eq!(
            serde_json::to_string(&ModelProfile::Qwen35Text08B).unwrap(),
            "\"qwen35_text08_b\""
        );
    }

    #[test]
    fn configured_api_still_requires_a_generation_probe() {
        let mut request = request(DeploymentTarget::Web);
        request.signals.api_configured = true;

        let plan = build_inference_backend_plan(host("linux"), request);

        assert_eq!(plan.recommended_backend, None);
        assert_eq!(plan.next_probe, Some(InferenceBackendId::OpenAiCompatible));
        assert_eq!(
            backend(&plan, InferenceBackendId::OpenAiCompatible).readiness,
            BackendReadiness::ProbeRequired
        );
    }

    #[test]
    fn generated_api_can_be_selected() {
        let mut request = request(DeploymentTarget::Web);
        request.signals.api_configured = true;
        request.signals.api_service_ready = true;

        let plan = build_inference_backend_plan(host("linux"), request);

        assert_eq!(
            plan.recommended_backend,
            Some(InferenceBackendId::OpenAiCompatible)
        );
    }
}
