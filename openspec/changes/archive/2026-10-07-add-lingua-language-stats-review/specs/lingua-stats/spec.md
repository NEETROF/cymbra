## ADDED Requirements

### Requirement: A device's daily statistics carry their language
A device SHALL send one daily statistic per day and studied language, each with its language, instead of sending every count as English.

#### Scenario: A day of English and Spanish
- **WHEN** a device that read English and Spanish today syncs
- **THEN** it sends an English statistic and a Spanish statistic for today, each with its own counts
