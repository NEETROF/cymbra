## ADDED Requirements

### Requirement: A daily statistic carries the native language of its device
The server SHALL carry the native language of the device as a field of each daily statistic, SHALL read an absent or empty value as `fr`, and SHALL store it as a value of the statistic's row, whose key stays (reader, UTC day, studied language, device). The last upsert of a day SHALL set the value, as it sets the counts. The consolidated read a reader sees SHALL be unchanged.

#### Scenario: An installed client that sends no native language
- **WHEN** a client built after the new-words counter and before this field upserts a day
- **THEN** the day is stored with the native language `fr`

#### Scenario: A Spanish-speaking reader's day
- **WHEN** a device whose native language is Spanish upserts a day of English reading
- **THEN** the row for (reader, day, `en`, device) holds `es` as its native language

#### Scenario: The native language changes during a day
- **WHEN** a device upserts a day as `fr`, then upserts the same day as `es`
- **THEN** the row holds `es` and the counts of the last upsert, and the day is counted once

#### Scenario: The reader's own statistics
- **WHEN** a reader reads their consolidated statistics
- **THEN** they are summed per (day, studied language) over their devices, as before, and carry no native language
