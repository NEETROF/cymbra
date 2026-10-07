## ADDED Requirements

### Requirement: A device's daily statistics carry its native language
A device SHALL keep, for each day and studied language, the native language under which the day was counted, the last one when it changed, and SHALL send it with the statistic. It SHALL send a statistic whose native language is not French only to a server whose data state says it stores the labels, and SHALL hold it otherwise. Statistics written before this requirement SHALL be read once as French.

#### Scenario: A French-native device
- **WHEN** a device whose native language is French syncs today's English statistic
- **THEN** the statistic carries the native language `fr`, and the server's row is as before

#### Scenario: A Spanish-native device
- **WHEN** a device whose native language is Spanish syncs today's English statistic with a server that stores the labels
- **THEN** the statistic carries `es`

#### Scenario: A server that predates the labels
- **WHEN** a Spanish-native device syncs with a server whose data state does not say it stores the labels
- **THEN** today's statistics are held, and sent at a later sync against a server that stores them

#### Scenario: Statistics written before the label
- **WHEN** a device updates with statistics written by the previous build
- **THEN** each is read once with the native language `fr`, and its counts are unchanged
