# 👾 Glitch Cache

An on-chain gacha simulator built on Stellar (Soroban). No hidden odds, no house edge. Everything runs entirely on the ledger.

## how it works
- **the faucet**: claim daily SC (stellar credits). 100/day, 500 on a 7-day streak.
- **the cases**: spend SC to roll 5 different boxes. drops range from common junk to 1% mythic artifacts.
- **the market**: pulled something trash? sell it back to the contract for instant liquidity to keep rolling.

## network
- **net**: stellar testnet
- **contract**: `CDLOTJMCKFC4EJUT3H3K3LQPJ46PG2B2S5TROXSP6OVMMN7OG6OU2TJG`

## CLI commands
How to interact with the live contract using the stellar CLI.

**1. seed the db (admin run once)**
`stellar contract invoke --id CDLOTJMCKFC4EJUT3H3K3LQPJ46PG2B2S5TROXSP6OVMMN7OG6OU2TJG --source <wallet> --network testnet -- init --admin <wallet>`

**2. get daily cash**
`stellar contract invoke --id CDLOTJMCKFC4EJUT3H3K3LQPJ46PG2B2S5TROXSP6OVMMN7OG6OU2TJG --source <wallet> --network testnet -- claim_daily --caller <wallet>`

**3. pull a box (case_id 1 to 5)**
`stellar contract invoke --id CDLOTJMCKFC4EJUT3H3K3LQPJ46PG2B2S5TROXSP6OVMMN7OG6OU2TJG --source <wallet> --network testnet -- open_case --caller <wallet> --case_id 1`

**4. check your stash**
`stellar contract invoke --id CDLOTJMCKFC4EJUT3H3K3LQPJ46PG2B2S5TROXSP6OVMMN7OG6OU2TJG --source <wallet> --network testnet -- get_inventory --user <wallet>`

**5. dump an item for SC**
`stellar contract invoke --id CDLOTJMCKFC4EJUT3H3K3LQPJ46PG2B2S5TROXSP6OVMMN7OG6OU2TJG --source <wallet> --network testnet -- sell_item --caller <wallet> --item_id <id>`