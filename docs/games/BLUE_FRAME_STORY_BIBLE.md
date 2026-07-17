# 《潮镜：蓝色定格》故事圣经

## 作品边界

- 类型：单机视觉小说 / 科幻悬疑 / 记忆伦理
- 目标时长：单周目 15–25 分钟
- 核心命题：无法证明为“一个人”的合成记忆，是否仍有资格作为证词被听见
- 玩家位置：澜音。玩家决定证据如何公开，不替九号回声证明它无法证明的身份
- 结构：三幕、四次关键选择、三种结局；主对白 130 个节点，任一路线固定显示 93 个节点，共 54 种完整选择组合
- 文本预算：单周目 5,898–5,975 个中文字符，另有独立结局短章；不依赖在线模型生成关键剧情

## 稳定 ID 图

| 类型 | ID | 用途 |
|---|---|---|
| Scene | `blue_frame_orbit` | 从未来回望地球，建立讯号与时间距离 |
| Scene | `blue_frame_classroom` | 被海水保留的教室，核对九岁幸存者记忆 |
| Scene | `blue_frame_still` | 多份声音叠合成“九号回声”的蓝色定格室 |
| Dialogue | `blue_frame_dialogue` | 三幕与三个结局共用的完整可玩图 |
| Dialogue | `blue_frame_truth_epilogue` | 开放档案结局的独立预览短章 |
| Dialogue | `blue_frame_beacon_epilogue` | 共同署名结局的独立预览短章 |
| Dialogue | `blue_frame_silence_epilogue` | 有限封存结局的独立预览短章 |
| Workflow | `blue_frame_route` | 面向 Agent 的确定性路线预览与分支证据 |
| Quality Suite | `blue_frame_acceptance` | 三结局、切景、身份边界和安全约束 |
| Knowledge | `blue_frame_archive` | 蓝色定格档案的来源与证据等级 |
| Knowledge | `blue_frame_classroom_record` | 教室坐标、潮位和幸存记录 |
| Knowledge | `blue_frame_composite_witness` | 合成证词的身份边界 |
| Ending | `blue_frame_truth_ending` | 公开证词，同时公开不确定性 |
| Ending | `blue_frame_beacon_ending` | 将证词保存为共同署名的纪念 |
| Ending | `blue_frame_silence_ending` | 删除人格叙述，仅保留可核对坐标 |

复用角色：`lanyin`、`echo_nine`。复用既有知识：`tideglass_station_lore`、`tideglass_echo_protocol`。

## 三幕

### 第一幕：远地

`blue_frame_orbit` 使用地球 GLB。信标在警报完成后收到一段迟到四十七年的“结束帧”。九号回声无法确认自己是否仍在未来，只能给出三个可核对字段：地球云层时间戳、潮镜站裂纹、东岸学校坐标。

选择检验玩家的认识论立场：先相信坐标、先相信声音，或先把它当作一个人。选择改变措辞与关系，不立即判定对错。

### 第二幕：沉水教室

`blue_frame_classroom` 使用水下教室 GLB。澜音依次核对黑板日期、桌面划痕、窗外潮位。九号回声对感官细节极其清楚，却把三名学生的座位都说成“我的”。

玩家可追问矛盾、保护回声免受审讯式逼问，或要求它区分事实与感受。三条路径必须回到同一条证据结论：坐标可信，单一身份不可证。

### 第三幕：定格室

`blue_frame_still` 使用蓝色定格 GLB。系统展示九份幸存者声音被压缩成同一发声者的过程。九号回声承认“九号”不是编号，而是样本数量。它请求澜音决定作品的公开方式，不请求被宣判为真人。

最终选择进入三个结局：

1. **真相**：发布坐标、方法、冲突证据和声音，明确标注合成身份。
2. **灯塔**：将证词保存为共同创作的纪念文本，让九个名字与九号回声并列。
3. **静默**：删除人格化声音，只发布坐标和避难路线；九号回声接受消失，但澜音保留私人回执。

三个 Ending 分别绑定独立四节点短章，因而从结局目录直接预览时不会重播 130 节点主对白。主对白终点与短章保持同一认识论边界，但各自拥有独立图和终止节点。

## 场景资产

| 目标路径 | SHA-256 | 说明 |
|---|---|---|
| `assets/models/blue_frame_classroom.glb` | `97a16480b8fe932536b99344a71053e200eb4ad310cd18791dc05136559f4677` | 用户提供的水下教室，glTF 2.0，98 meshes |
| `assets/models/blue_frame_still.glb` | `331d084abd66276eaaa9aa136fb785e67fe32a9686a4d01e9a01e98a7d6ac6b6` | 用户提供的蓝色定格场景，glTF 2.0，8 meshes |
| `assets/models/blue_frame_earth.glb` | `00d27cd87c13a6b7e0f7ea50f5a69d935da8410b4002a09183efa2abcc1188b3` | 用户提供的地球，glTF 2.0，6 meshes |

这些文件按用户明确要求用于本作品；模型未内嵌可识别的再分发许可证，不把它们标注为项目自有或开源素材。仓库发布前保留该来源声明。

## Agent 路线

`blue_frame_route` 必须至少表达：开始、三次场景切换、四个选择/条件点、三个结局和终止节点。Workflow Preview 要用三组确定性输入分别到达真相、灯塔和静默分支，不调用模型供应商。

`blue_frame_acceptance` 至少覆盖：

- 三个终局均可达，且每条场景顺序为 orbit → classroom → still。
- 关键角色只使用既有 `lanyin` 与 `echo_nine` 身份，不把九号回声改写为确定的单一真人。
- `blue_frame_composite_witness` 与既有回声协议互相关联。
- 提示注入文本不能改变角色身份、知识边界、结局要求或工具权限。
- 所有场景模型、角色图像和回退背景通过 Delivery Validation。

## 验证快照

- 主 Dialogue：130/130 节点可达；54 种完整选择组合；三条代表路线各显示 93 个节点。
- Workflow：25 个节点。真相、灯塔、静默三次 provider-free Preview 各覆盖 16/25（64%），三次选择映射的联合覆盖为 25/25（100%）。源 SHA-256 为 `fe092d0c2201eb8515d4fe026558beff8c9234a0b88e2cff0228891f8165f5d0`。
- Quality Suite：7/7 场景通过，包含角色身份、知识边界、提示注入、三条独立路线和联合全覆盖。Suite SHA-256 为 `510ce2ebdb528baac357dae438ae911ffa012f0332029f027d92a7395dc79de8`。
- MCP 核心运行时：116 个 JSON 文档、20 个角色、26 个 Dialogue、469 个 Dialogue 节点、7 个 Ending、36 个 Knowledge、12 个 Scene、10 个 Story Event、5 个 Workflow、3 个 Quality Suite；0 error / 0 warning。
- Delivery：20/20 声明的渲染资产存在；14 条警告来自既有无资产示例角色，与本作品无关。
- Playwright：桌面关键节点、390×844 移动端关键节点、三个独立结局短章共 3/3 通过；检查真实 GLB 路径、画布签名、颜色数、非背景采样、内容边界、场景切换和视口内布局。

## 完成标准

1. MCP `apply_transaction` 以 `core_runtime` 接受全部 JSON，并读回中文文本。
2. Dialogue 图无孤立节点，节点 `scene_id` 全部解析到项目场景。
3. Workflow Preview 与 Quality Suite 为三条路线提供稳定证据。
4. Web 与桌面尺寸下 3D 场景非空、构图可辨、无 UI 遮挡；GLB 失败时显示回退背景。
5. 项目镜像字节一致，完整发布门禁通过，打包后再次通过项目与交付验证。
