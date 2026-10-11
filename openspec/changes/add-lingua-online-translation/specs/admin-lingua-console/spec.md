## ADDED Requirements

### Requirement: The console shows online translation usage and its estimated cost
The Lingua console SHALL show, over its day window, the number of online translations answered, failed and refused at the daily limit, and their estimated cost, in total, per day, per provider and per pair.
The figures SHALL come from daily totals stored by the translation service — requests, input and
output tokens, and the estimated cost computed from them at the time of the request — and SHALL
carry no account and no text, like every figure of this console. The cost SHALL be labelled as an
estimate in US dollars, computed from per-provider prices that an operator can change at runtime;
a price change SHALL apply to requests made after it and SHALL NOT rewrite past days.

#### Scenario: Reading a week of usage
- **WHEN** an admin of Lingua opens the console on the last seven days
- **THEN** it shows the online translations answered and their estimated cost for the week, per day, and split by provider and by pair

#### Scenario: A provider price changes
- **WHEN** an operator changes a provider's price
- **THEN** requests made after the change are costed at the new price, and earlier days keep their cost

#### Scenario: Nothing personal
- **WHEN** an admin reads the online translation figures
- **THEN** no account, handle, sentence or translation appears anywhere in them
