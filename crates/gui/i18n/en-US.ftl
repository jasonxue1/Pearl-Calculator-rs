app-title = Pearl Calculator
settings-button = ⚙ Settings
language-label = Language
language-option-english = English
language-option-zh-cn = 简体中文

tab-calculation = Calculation
tab-simulation = Simulation
tab-convert = Convert

input = Input
output = Output
config-path = Config Path

calculation-parameters = Calculation Parameters
target-x = Target X
target-z = Target Z
max-tnt-red-optional = Max TNT Red (optional)
max-tnt-blue-optional = Max TNT Blue (optional)
max-error-optional = Max Error (optional)
max-time-optional = Max Time (optional)
show-first-optional = Show First (optional)
dimension = Dimension
dimension-overworld = Overworld
dimension-nether = Nether
dimension-end = End
run-calculation = Run Calculation
no-calculation-results = No calculation results.
calculation-finished = Calculation finished. { $count } result(s).

simulation-parameters = Simulation Parameters
direction-range = Direction (0..=3)
time-optional = Time (optional)
to-end-time-optional = To End Time (optional)
run-simulation = Run Simulation
end-portal-position = End portal position
final-position = Final position

convert-parameters = Convert Parameters
convert-rb-label = RB
convert-code-label = Code
run-rb-to-code = RB ↓ Code
run-code-to-rb = Code ↑ RB
code-input-hint = Code string
convert-output-hint = Use the two buttons on the left to convert and overwrite the other field.

header-time = Time
header-dir = Dir
header-red = Red
header-blue = Blue
header-error = Error
header-pos = Pos (x, y, z)
header-to-end = To End
header-portal = Portal (x, y, z)
header-gt = GT
header-vel = Vel (x, y, z)
header-yaw = Yaw
header-dim = Dim

status-success = success
error-prefix = error:

core-error-unsupported-config-version = Unsupported config version: { $version }
core-error-invalid-direction-vector = Invalid direction vector: [{ $x }, { $y }]
core-error-invalid-direction-combination = Invalid direction combination sum: ({ $x }, { $y })
core-error-duplicate-direction-quadrant = Duplicate direction quadrant: { $quadrant }
core-error-simulation-time-zero = Simulation time must be greater than 0
core-error-to-end-time-after-end = to_end_time ({ $to_end_time }) cannot be greater than total time ({ $time })
core-error-end-portal-teleport-from-end = Cannot trigger end-portal teleport when already in End
core-error-unimplemented = Unimplemented feature: { $feature }
core-error-unsupported-dimension = Unsupported dimension { $dimension } in { $context }
core-error-invalid-max-tnt-arg-count = Invalid max-tnt argument count: { $count } (expected 0..=2)
core-error-invalid-cap-bit = Cap bit index out of range: { $bit } (must be 1..={ $max })
core-error-duplicate-cap-bit = Duplicate cap bit index in one cap group: { $bit }
core-error-overlapping-cap-bit = Cap bit index overlaps across groups: { $bit }
core-error-code-length-mismatch = Code length mismatch: expected { $expected } bits from rule, got { $actual }
core-error-mixed-cap-kinds = All bits in one cap group must have the same type
core-error-direction-out-of-range = Direction value out of range: { $value } (must be 0..=3)
core-error-value-overflow = Numeric overflow while accumulating code counts
core-error-no-exact-encoding = Cannot encode exact RB value: direction={ $direction }, red={ $red }, blue={ $blue }

config-error-read-failed = Failed to read config '{ $path }': { $source }
config-error-parse-json-failed = Failed to parse config json '{ $path }': { $source }
parse-error-must-be = { $field } must be a valid { $expected }
error-max-red-blue-pair = Max TNT Red and Max TNT Blue must both be empty or both provided
error-direction-range = Direction must be in range 0..=3
error-code-empty = Code cannot be empty
error-code-invalid-char = Invalid code character at position { $position }: '{ $char }'
parse-type-integer = integer
parse-type-number = number
