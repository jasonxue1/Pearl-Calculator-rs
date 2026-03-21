app-title = 珍珠炮计算器
settings-button = ⚙ 设置
language-label = 语言
language-option-english = English
language-option-zh-cn = 简体中文
settings-config-label = 配置
settings-config-none = （未选择）
settings-import-config = 导入配置
settings-import-conflict-title = 配置名冲突
settings-import-conflict-message = '{ $name }' 已存在。
settings-import-conflict-rename-label = 新名称
settings-import-cancel = 取消
settings-import-rename = 重命名
settings-import-overwrite = 覆盖

tab-calculation = 计算
tab-simulation = 模拟
tab-convert = 转换

input = 输入
output = 输出
config-path = 配置路径

calculation-parameters = 计算参数
target-x = 目标 X
target-z = 目标 Z
max-tnt-red-optional = 最大红 TNT（可选）
max-tnt-blue-optional = 最大蓝 TNT（可选）
max-error-optional = 最大误差（可选）
max-time-optional = 最大时间（可选）
show-first-optional = 仅显示前 N 项（可选）
dimension = 维度
dimension-overworld = 主世界
dimension-nether = 下界
dimension-end = 末地
run-calculation = 开始计算
no-calculation-results = 没有计算结果。
calculation-finished = 计算完成，共 { $count } 条结果。

simulation-parameters = 模拟参数
direction-range = 方向 (0..=3)
time-optional = 时间（可选）
to-end-time-optional = 到末地时间（可选）
run-simulation = 开始模拟
end-portal-position = 末地门坐标
final-position = 最终坐标

convert-parameters = 转换参数
convert-rb-label = RB
convert-code-label = Code
run-rb-to-code = RB ↓ Code
run-code-to-rb = Code ↑ RB
code-input-hint = Code 字符串
convert-output-hint = 使用左侧两个按钮进行双向转换，会覆盖另一个输入框。

header-time = 时间
header-action = 操作
header-dir = 方向
header-red = 红
header-blue = 蓝
header-error = 误差
header-pos = 坐标 (x, y, z)
header-to-end = 到末地
header-portal = 末地门 (x, y, z)
header-gt = 刻
header-vel = 速度 (x, y, z)
header-yaw = 偏航角
header-dim = 维度
calculation-code-output-empty = 点击上方某一行最左侧的图标按钮后，这里会显示对应 Code。

status-success = 成功
error-prefix = 错误：

core-error-unsupported-config-version = 不支持的配置版本：{ $version }
core-error-invalid-direction-vector = 无效的方向向量：[{ $x }, { $y }]
core-error-invalid-direction-combination = 无效的方向组合和：({ $x }, { $y })
core-error-duplicate-direction-quadrant = 重复的方向象限：{ $quadrant }
core-error-simulation-time-zero = 模拟时间必须大于 0
core-error-to-end-time-after-end = 到末地时间（{ $to_end_time }）不能大于总时间（{ $time }）
core-error-end-portal-teleport-from-end = 已在末地时，不能再次触发末地门传送
core-error-unimplemented = 未实现功能：{ $feature }
core-error-unsupported-dimension = 在 { $context } 中不支持维度 { $dimension }
core-error-invalid-max-tnt-arg-count = max-tnt 参数数量无效：{ $count }（期望 0..=2）
core-error-invalid-cap-bit = cap 位索引越界：{ $bit }（必须在 1..={ $max }）
core-error-duplicate-cap-bit = 单个 cap 组内存在重复位索引：{ $bit }
core-error-overlapping-cap-bit = cap 位索引在组间重叠：{ $bit }
core-error-code-length-mismatch = code 长度不匹配：根据规则应为 { $expected } 位，实际为 { $actual } 位
core-error-mixed-cap-kinds = 单个 cap 组内所有位必须是同一类型
core-error-direction-out-of-range = 方向值超出范围：{ $value }（必须是 0..=3）
core-error-value-overflow = 累计 code 计数时发生数值溢出
core-error-no-exact-encoding = 无法精确编码 RB 值：direction={ $direction }, red={ $red }, blue={ $blue }

config-error-read-failed = 读取配置失败 '{ $path }'：{ $source }
config-error-parse-json-failed = 解析配置 JSON 失败 '{ $path }'：{ $source }
config-error-store-unavailable = 配置目录不可用
config-error-no-selected = 请先选择一个配置
config-error-selected-not-found = 找不到已选择的配置：{ $name }
config-error-empty-default = 配置文件为空：{ $path }
parse-error-must-be = { $field } 必须是有效的{ $expected }
error-max-red-blue-pair = 最大红 TNT 和最大蓝 TNT 必须同时为空或同时填写
error-direction-range = 方向必须在 0..=3 范围内
error-code-empty = Code 不能为空
error-code-invalid-char = Code 在第 { $position } 位包含无效字符：'{ $char }'
settings-error-invalid-file-name = 文件名无效
settings-error-empty-config-name = 配置名不能为空
settings-error-target-exists = 目标配置已存在
settings-error-target-exists-name = 配置 '{ $name }' 已存在
parse-type-integer = 整数
parse-type-number = 数字
