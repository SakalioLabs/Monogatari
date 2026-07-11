# Inference Backend Matrix

Last verified: 2026-07-11

This document defines the model/runtime evidence, automatic selection policy,
and staged rollout process for Monogatari inference backends. It is deliberately
strict: detecting a GPU, driver, DLL, command, or execution provider does not
make a backend ready. A backend becomes `ready` only after the exact model
profile completes a generation probe in the current runtime profile.

## Readiness states

| State | Meaning | May auto-select? |
| --- | --- | --- |
| `ready` | Exact model initialized and generated; required API/stream probes passed | Yes |
| `probe_required` | Runtime and model are present, but the exact model has not generated | No |
| `setup_required` | Runtime, model, or service profile still needs installation | No |
| `blocked` | A reproduced incompatibility prevents this model/runtime pairing | No |
| `unavailable` | Backend does not apply to this target or was not detected | No |

The versioned report schema is `monogatari-inference-backend-plan/v1`. The
selection core lives in `rust-engine/crates/ai/src/backend_selection.rs`, is
exposed by the Tauri command `get_inference_backend_plan`, and is rendered in
Settings > Diagnostics.

## Model and 3D contract

The default text profile is:

- Model family: Qwen3.5 0.8B
- Web artifact: `onnx-community/Qwen3.5-0.8B-Text-ONNX`, Q4
- Native/service artifact: Qwen3.5 0.8B GGUF Q4_K_M or an explicitly exported
  ORT GenAI directory
- Required output contract: chat text, bounded generation, complete stream
  termination, no private reasoning persistence

Qwen3.5 is a vision-language model, not a raw 3D mesh model. It does not consume
GLB, GLTF, vertices, materials, bones, or animation tracks as native inputs.
Monogatari can use a 3D character with Qwen3.5 in two separate ways:

1. Load and animate GLB/GLTF through `CharacterModelView.vue` and Three.js.
2. Render a frame or thumbnail and pass that raster image to a compatible
   vision profile.

Geometry-aware reasoning would require a separate mesh encoder, structured scene
description, or tool adapter. Rendering support is verified separately with the
Khronos Fox GLB fixture: the checked-in character record loads the real animated
glTF 2.0 model, normalizes arbitrary source units, frames it for desktop/mobile
aspects, and exposes deterministic canvas state for visual and pixel probes.

3D renderer evidence on the Windows test host:

| Viewport | State | Pixel evidence | Framing evidence |
| --- | --- | --- | --- |
| 1440x900 desktop | `ready`, 3 animation clips, textured, motion detected | 67 sampled colors, luminance 13-195, 89 non-background samples | 24x24 content bounds `5,3,14,21`; margin on every edge; no console errors/warnings |
| 375x812 mobile | `ready`, 3 animation clips, textured, motion detected | 61 sampled colors, luminance 13-194, 75 non-background samples | preview rect `17,465,353,765`; content bounds `6,3,13,21`; no overflow or console errors/warnings |

The unmodified GLB is 162,852 bytes with SHA-256
`d97044e701822bac5a62696459b27d7b375aada5de8574ed4362edbba94771f7`.
Its CC0/CC BY attribution ships beside the model in both data roots.

## Verified evidence

Test host:

- Windows 11 Pro, build 26200
- Intel Core i7-14700KF
- NVIDIA GeForce RTX 3060 12 GB
- WSL2 Ubuntu 24.04.1, kernel 6.6.87.2

| Profile | Runtime | Result | Evidence |
| --- | --- | --- | --- |
| Web/PWA | Transformers.js 4.2.0 + ORT WebGPU + Qwen3.5 text Q4 | `ready` | Model initialized on WebGPU and produced a complete Chinese reply; production package and runtime contract passed |
| Windows ONNX | ORT DirectML 1.24.4 + ONNX Community text graph | `blocked` | Session load failed because `com.microsoft:CausalConvWithState` was not registered |
| Windows GenAI | ORT GenAI DirectML 0.13.1 + ORT DirectML 1.24.4 | `blocked` | Export completed, but runtime failed on the same missing hybrid operator |
| Windows WinML | ORT GenAI WinML 0.14.1 + WindowsML ORT 1.24.6 | `blocked` | Qwen3.5 text export still failed on missing `CausalConvWithState` |
| Windows WinML | ORT GenAI WinML 0.14.1 + WindowsML ORT 1.27.0 | `blocked` | Operator was recognized, then DML graph capture failed because not all nodes were partitioned to DML |
| Windows CPU | ORT GenAI WinML 0.14.1 + ORT 1.27.0 + Qwen3.5 text INT4 | `ready` for diagnostic use | Loaded in 0.835 s; generated 26 Chinese tokens in 1.442 s |
| Linux CPU CLI | llama.cpp b9957 + Qwen3.5 0.8B Q4_K_M | `ready` | 237.8 prompt tok/s and 29.7 generation tok/s; complete greeting |
| Linux CPU service | llama-server b9957 OpenAI-compatible API | `ready` | `/health`, non-streaming chat completion, and SSE stream passed; stream emitted 12 data frames and `[DONE]` |
| WSL Vulkan | llama.cpp b9957 Vulkan build | `unavailable` on this host | Vulkan loader exists, but `llama-cli --list-devices` returned no devices |
| Windows llama.cpp | b9957 CPU and Vulkan release binaries | `blocked` on this host/build | Both exited with Windows access violation `0xC0000005` before a model probe |

The Linux service probe returned a normal OpenAI-compatible response and can be
consumed by the existing Rust `APIEngine` without a new inference protocol.
Observed server timing for one 32-token request was 256.46 prompt tok/s and
25.79 generation tok/s. Streaming emitted valid `data:` frames and a terminal
`[DONE]` marker.

## Windows conclusion

The current Rust `ONNXEngine` is intentionally small. It supports a single
full-sequence causal-LM graph with `input_ids`, optional mask/position/type IDs,
and float32 logits. Qwen3.5 uses a hybrid decoder with:

- six full-attention KV-cache layers;
- eighteen convolution state tensors;
- eighteen recurrent state tensors;
- fused `CausalConvWithState` and linear-attention operators.

That contract cannot be driven safely by the current full-sequence loop. Adding
ad hoc input-name handling would still leave cache allocation, state sharing,
provider placement, and model-version compatibility unresolved. The native
Qwen3.5 path should use ORT GenAI through a sidecar or maintained FFI boundary,
not a hand-written hybrid cache loop.

The successful ORT GenAI export required the 0.14 builder and:

```text
precision=int4
execution_provider=dml
exclude_embeds=false
int4_block_size=32
```

`exclude_embeds=false` produces the standalone `qwen3_5_text` runtime type and
keeps `input_ids` in the decoder. Older output was classified as multimodal and
incorrectly required vision and embedding sessions.

DirectML itself is now in sustained engineering. New Windows ONNX deployment
work should target Windows ML, which can acquire and register hardware-specific
execution providers. This does not remove model optimization or model-level
verification requirements. On the tested host, upgrading WindowsML ORT from
1.24.6 to 1.27.0 moved the failure from an unknown operator to an invalid DML
graph capture, so WinML Qwen3.5 remains blocked rather than release-ready.

Current Windows policy:

1. Prefer verified WebGPU for Web/PWA.
2. Probe a pinned llama.cpp CUDA or Vulkan sidecar per release before enabling it.
3. Keep ORT GenAI CPU as a diagnostic/fallback profile, not the default package.
4. Keep WinML GenAI and linked DirectML behind explicit model-level gates.
5. Use a validated OpenAI-compatible endpoint when no local profile is ready.

## Automatic selection policy

The planner accepts host detection plus completed probe signals. Detection may
create `probe_required`; only a completed model probe creates `ready`.

### Web

1. WebGPU
2. Generation-verified OpenAI-compatible API

### Windows desktop

1. WebGPU with exact-model generation proof
2. WinML GenAI with exact-model generation proof
3. llama.cpp local service with health, generation, and SSE proof
4. linked DirectML with a compatible full-sequence model proof
5. generation-verified OpenAI-compatible API

For Qwen3.5, WinML and linked DirectML are currently marked `blocked`; they are
removed from the fallback order automatically.

### Linux desktop

1. llama.cpp service, selecting only a backend shown by `--list-devices`
2. WebGPU when running the Web/PWA package
3. generation-verified OpenAI-compatible API

### macOS desktop

1. llama.cpp Metal service after model generation and SSE probes
2. MLX-LM after exact conversion and generation probes
3. WebGPU when running the Web/PWA package
4. generation-verified OpenAI-compatible API

### Linux server

1. vLLM for a verified high-concurrency profile
2. SGLang for a verified high-concurrency or vendor-specific profile
3. llama.cpp for compact models, edge servers, or simple deployment
4. generation-verified upstream OpenAI-compatible API

vLLM and SGLang are service profiles. They are not bundled into the desktop
application. The application consumes their OpenAI-compatible APIs through the
existing `APIEngine`.

## Staged adaptation workflow

Every backend follows the same promotion stages. A backend may not skip a stage.

### Stage 0: lock the profile

Record:

- backend and runtime version;
- model repository, revision, format, precision, and SHA-256;
- driver and toolkit versions;
- OS, architecture, GPU/NPU model, and memory;
- launch arguments and environment overrides;
- license and redistribution decision.

Do not use floating container tags, unpinned nightly wheels, or model `main`
revisions in release profiles.

### Stage 1: detect capability

Detection is read-only and fast:

- WebGPU: secure context plus `navigator.gpu` adapter;
- CUDA: driver visibility, then runtime/device probe;
- Vulkan: `vulkaninfo` and llama.cpp `--list-devices`;
- ROCm: `/dev/kfd`, `rocminfo`, supported GFX target;
- MUSA: vendor driver utility and toolkit paths;
- Intel: `sycl-ls`, OpenVINO device enumeration, or Vulkan device listing;
- Metal: Apple Silicon/macOS plus backend device listing;
- WinML: Windows ML runtime and execution-provider catalog;
- service: loopback or configured endpoint health check.

Do not infer Vulkan readiness from `libvulkan` alone. The tested WSL host is a
concrete counterexample: the loader was installed and no Vulkan device was
available.

### Stage 2: initialize the runtime

- Load the exact runtime libraries or image.
- Enumerate the selected device.
- Fail if the requested accelerator silently falls back to CPU, unless the
  profile explicitly allows CPU.
- Capture runtime and provider versions in the probe report.

### Stage 3: run the exact model

Required prompts:

1. deterministic ASCII greeting;
2. deterministic Chinese greeting;
3. 512-token context continuity case;
4. stop-sequence case;
5. maximum configured output boundary;
6. cancellation and timeout case.

Validate non-empty text, terminal stop reason, token counts, no NaN/Inf logits,
and no leaked private reasoning in stored chat history.

### Stage 4: validate the service contract

For sidecars and server frameworks:

- `GET /health` succeeds;
- `POST /v1/chat/completions` succeeds;
- streaming emits valid SSE frames;
- stream ends with `[DONE]`;
- disconnect cancels work;
- malformed and provider error frames fail closed;
- localhost HTTP is accepted; non-loopback plaintext HTTP is rejected;
- secrets are never written to project settings or logs.

### Stage 5: benchmark

Record cold load, warm load, prompt tok/s, generation tok/s, first-token latency,
peak host RAM, peak device memory, sustained power, and 30-minute stability.
Run at context lengths 512, 2K, 8K, and the product maximum. Do not publish the
model's theoretical context length as an application guarantee.

### Stage 6: package

Keep heavy vendor runtimes outside the core desktop package:

- one signed sidecar/archive or container image per backend profile;
- checksummed manifest and explicit compatible device list;
- install-on-demand with user-visible size and license;
- process supervision, health timeout, log rotation, and clean shutdown;
- no model weights in Git;
- model cache outside project content and export archives.

### Stage 7: promote

Promote only after CI/static checks and a hardware-lab result both pass. Store a
last-known-good report keyed by runtime version, model SHA-256, device identity,
and driver version. Any key change returns the backend to `probe_required`.

## Runtime-specific flows

### llama.cpp: CUDA, Vulkan, Metal, HIP, SYCL, CPU

Use llama.cpp as the primary compact local service candidate because one GGUF
model can be exposed through the same OpenAI-compatible API on each platform.

1. Pin a llama.cpp commit/release and archive checksum.
2. Install or build only the required backend.
3. Run `llama-cli --list-devices`.
4. Start with an explicit device/backend and small context.
5. Confirm logs show the intended offload and no silent CPU fallback.
6. Run CLI greeting, server health, non-streaming, and SSE probes.
7. Benchmark and record the exact launch profile.

Do not assume the newest prebuilt archive is healthy. The b9957 Windows CPU and
Vulkan archives crashed on the tested host while the Linux archive passed.

### vLLM

Use vLLM for Linux servers where batching, concurrency, tensor/data parallelism,
prefix caching, and operational telemetry justify the larger deployment.

1. Start from an official pinned image or wheel set.
2. Verify the exact Qwen3.5 architecture is registered in that release.
3. Pin Transformers and PyTorch versions from the image lockfile.
4. Start text-only mode when vision is not required.
5. Begin with eager/single-GPU settings and a short context.
6. Add CUDA graphs, parallelism, quantization, and long context one at a time.
7. Run OpenAI API, reasoning parser, tool-call, cancellation, and load probes.
8. Maintain separate NVIDIA and ROCm profiles.

Qwen3.5 support changed quickly across vLLM releases. A version label alone is
not sufficient evidence; the exact image and model must generate.

### SGLang

Use SGLang when its scheduler, structured generation, or vendor integration is
demonstrably better for the target service.

1. Pin a release/image and model revision.
2. Verify Qwen3.5 architecture and tokenizer compatibility.
3. Start one GPU, eager mode, and short context.
4. Validate OpenAI-compatible chat and streaming.
5. Add attention kernels, graph capture, parallelism, and long context in
   isolated steps.
6. Keep NVIDIA, ROCm, and MUSA artifacts separate.

Do not make SGLang the generic desktop fallback. Its value is in a managed
service, and its vendor paths carry large toolkit and image dependencies.

### macOS Metal and MLX

Preferred experiments:

1. llama.cpp Metal with Q4_K_M GGUF and an OpenAI-compatible loopback server;
2. MLX-LM with a pinned conversion and local server/sidecar adapter.

For each Apple Silicon generation, record macOS version, chip, unified memory,
runtime version, model hash, and Metal/MLX device evidence. Validate memory
pressure and thermal stability on the lowest supported memory tier. Intel Macs
must use a separate CPU policy; do not label them Metal-ready by association.

### AMD ROCm

Maintain two independent candidates:

- llama.cpp HIP for compact GGUF deployment;
- vLLM or SGLang ROCm for managed serving.

Flow:

1. Record GPU PCI ID and GFX architecture.
2. Select a supported ROCm version and host kernel.
3. Verify `/dev/kfd`, render group access, and `rocminfo`.
4. Pin the ROCm image/wheel and PyTorch build.
5. Run a one-layer/device allocation smoke before downloading the model.
6. Run exact-model generation, SSE, memory, and stability gates.
7. Package ROCm separately from CUDA and never load both stacks in one process.

APUs, consumer Radeon cards, and data-center Instinct GPUs need separate result
rows even when they share a nominal ROCm version.

### Intel GPU/NPU

Candidates:

- llama.cpp SYCL for Intel GPU GGUF inference;
- OpenVINO/GenAI for supported exported models;
- Vulkan only when device enumeration and exact-model offload pass.

Flow:

1. Record integrated/discrete GPU and driver versions.
2. Verify `sycl-ls` or OpenVINO device enumeration.
3. Pin oneAPI/OpenVINO runtime and compiler versions.
4. Run a device allocation smoke.
5. Run exact-model generation and confirm device placement.
6. Benchmark against CPU before promotion; an available iGPU is not always the
   faster or more memory-efficient choice.

### Moore Threads MUSA

MUSA must remain an optional, separately built server profile until tested on
physical Moore Threads hardware.

1. Provision a dedicated MUSA worker with a supported kernel and driver.
2. Record GPU model, driver, MUSA toolkit, MCC, and PyTorch-MUSA versions.
3. Validate the vendor device utility and a tensor allocation smoke.
4. Select a pinned SGLang/vendor fork or another officially supported runtime.
5. Build in a dedicated container; do not mix CUDA and MUSA libraries.
6. Run Qwen3.5 architecture import before downloading full weights.
7. Run exact-model generation, SSE, memory, and 30-minute stability probes.
8. Review redistribution terms for drivers, toolkit libraries, and images.
9. Publish a MUSA profile only after CI on the dedicated worker passes.

Upstream support trackers are planning evidence, not release evidence. Until the
hardware probe passes, the planner reports MUSA as detected/setup-required or
probe-required, never ready.

## Probe report requirements

A persisted hardware-lab result should include at least:

```json
{
  "schema": "monogatari-inference-probe/v1",
  "backend": "llama_cpp",
  "runtime_version": "b9957-c4ae9a88f",
  "model_profile": "qwen35_text08_b",
  "model_sha256": "...",
  "os": "linux",
  "architecture": "x86_64",
  "device": {
    "kind": "cpu",
    "name": "Intel Core i7-14700KF",
    "driver": null
  },
  "probes": {
    "model_loaded": true,
    "generated_text": true,
    "chat_completion": true,
    "sse_done": true,
    "silent_cpu_fallback": false
  }
}
```

The report must not include API keys, passwords, authorization headers, raw
private prompts, or user project paths.

## Release gates

- Backend selection unit tests pass.
- Tauri command serializes stable snake_case backend IDs.
- Settings diagnostics render without overflow at desktop/tablet/mobile widths.
- WebGPU production build contains the pinned runtime contract and Asyncify WASM.
- Every `ready` native/service row has a hardware-lab probe artifact.
- Qwen3.5 DirectML and WinML remain blocked until their reproduced failures are
  replaced by exact-model success evidence.
- No unverified backend is selected automatically.
- Model weights and temporary exports remain outside Git.
- The animated GLB fixture passes header/hash/attribution checks and desktop/mobile framebuffer probes.

## Primary references

- [Qwen3.5 0.8B](https://huggingface.co/Qwen/Qwen3.5-0.8B)
- [ONNX Community Qwen3.5 text model](https://huggingface.co/onnx-community/Qwen3.5-0.8B-Text-ONNX)
- [ONNX Runtime GenAI](https://github.com/microsoft/onnxruntime-genai)
- [ONNX Runtime DirectML provider](https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html)
- [Windows ML overview](https://learn.microsoft.com/en-us/windows/ai/new-windows-ml/overview)
- [Windows ML execution providers](https://learn.microsoft.com/en-us/windows/ai/new-windows-ml/initialize-execution-providers)
- [llama.cpp](https://github.com/ggml-org/llama.cpp)
- [vLLM GPU installation](https://docs.vllm.ai/en/latest/getting_started/installation/gpu/)
- [MLX-LM](https://github.com/ml-explore/mlx-lm)
- [SGLang AMD platform guide](https://github.com/sgl-project/sglang/blob/main/docs/platforms/amd_gpu.md)
- [SGLang MUSA support tracker](https://github.com/sgl-project/sglang/issues/16565)
- [Khronos Fox glTF sample and attribution](https://github.com/KhronosGroup/glTF-Sample-Assets/tree/main/Models/Fox)
