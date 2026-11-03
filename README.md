# Mini-Launchpad

Проект модуля 7: фабрика SPL-токенов на Solana с комиссией в SOL, рассчитанной по
USD-цене из собственного on-chain оракула.

Пользователь вызывает `create_token`, фабрика берёт актуальную цену SOL/USD из оракула,
пересчитывает комиссию из USD в лампорты, переводит её в казну, создаёт Mint и ATA через
CPI и минтит начальный supply.

## Развёрнуто в Devnet

| Что                                       | Адрес                                                                                                                                             |
|-------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------|
| Программа `oracle` (Program ID)           | [`2kNW9ZJUzCyCZeZi1y6nffir39u2QXh845jf3Nc6gt5X`](https://explorer.solana.com/address/2kNW9ZJUzCyCZeZi1y6nffir39u2QXh845jf3Nc6gt5X?cluster=devnet) |
| Программа `minter` (Program ID)           | [`7Rku9YJHdjyo2PCcHyp8T9jEUrgDZFMxe9djLhync6t`](https://explorer.solana.com/address/7Rku9YJHdjyo2PCcHyp8T9jEUrgDZFMxe9djLhync6t?cluster=devnet)   |
| PDA `OracleState` (`seeds = ["oracle"]`)  | [`8SVbLbWtGTZ3QRXJg1EjdNCFyRyG8mZxr438rhcq5nRN`](https://explorer.solana.com/address/8SVbLbWtGTZ3QRXJg1EjdNCFyRyG8mZxr438rhcq5nRN?cluster=devnet) |
| PDA `MinterConfig` (`seeds = ["config"]`) | [`7XuVrNTizkG1KsibqKRYZabhTFgQ1dC1hN7Nf4HQW36a`](https://explorer.solana.com/address/7XuVrNTizkG1KsibqKRYZabhTFgQ1dC1hN7Nf4HQW36a?cluster=devnet) |
| Казна (получатель комиссий)               | [`Dz4rFsjHmYKr6rZKjpx2AETUmVvzk3WbZudidvUQEomR`](https://explorer.solana.com/address/Dz4rFsjHmYKr6rZKjpx2AETUmVvzk3WbZudidvUQEomR?cluster=devnet) |
| Upgrade authority = admin обеих программ  | `9PaJcWyiDcpAZYBDssjXi1Y6Z9xViPPJTRYM3WqhU1sh`                                                                                                    |

IDL обеих программ загружен on-chain (Program Metadata Program), поэтому Explorer
показывает имена инструкций и аргументы.

### Транзакции создания токенов

| Токен | decimals | supply     | Транзакция                                                                                                                                          | Mint                                                                                                         |
|-------|----------|------------|-----------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------|
| #1    | 6        | 1 000      | [3VtvjL44…](https://explorer.solana.com/tx/3VtvjL44N49yqx4CfMBn8M7PfxMokg5SFJRXn4HsnB6MtE5nJWSyUmMZFexj7uWWNb4xos11e8RzLG1qVoyvyXhi?cluster=devnet) | [DWo8BCcJ…](https://explorer.solana.com/address/DWo8BCcJ3CrxEijPQmtiLsiHm4RQ6RGeAaspQ7VkZSTr?cluster=devnet) |
| #2    | 9        | 21 000 000 | [56abNSyX…](https://explorer.solana.com/tx/56abNSyXW3Eoeo7Fx6PEsAwzcxLLpRmnbvVsh9PAmJ9DpRzTxoG9p1R2c2qkAmnemQAysWo65vHMVfyktxBtXGoS?cluster=devnet) | [8sK98ZXi…](https://explorer.solana.com/address/8sK98ZXiT9Akq3CTjr4Rt8HyfB9EwQ2LrFC2DWmL9G5r?cluster=devnet) |
| #3    | 0        | 100        | [5oqSk4bK…](https://explorer.solana.com/tx/5oqSk4bKhqBLrjdCoRghVHWL8hgNLoziBAHGYMue5DP5gVUc1X8xuRNgMSnssgkFCKoQRsTwkhQzgH4CLj9NPBuQ?cluster=devnet) | [7SHv6vje…](https://explorer.solana.com/address/7SHv6vje2c8Tz8PgJeth1WX6tU9v3JHqmUS2gxH2R3DY?cluster=devnet) |

В каждой транзакции видны CPI в System Program (комиссия в казну и создание аккаунтов),
SPL Token (`InitializeMint2`, `MintTo`), Associated Token Program (создание ATA) и
событие `TokenCreated` в логах.

### Инициализация и работа оракула

- Инициализация
  оракула: [3zf7LsCH…](https://explorer.solana.com/tx/3zf7LsCHZXGEj6HBwTdzstyQvghthX6tu7ULNQ66ry78QZjFeUU8pJhTSgpPYjjmWL7TbUWTPMy6uHnPP54ypH9y?cluster=devnet)
- Первая цена (150
  USD/SOL): [sAkZPUZN…](https://explorer.solana.com/tx/sAkZPUZNxstYd5XD9S5iKUWVzBNW9GdMFAvVzvLFE1Mg5URSAXiFnhvmkfjxSfqZ52SqFBdHvcky7HfXuk8LKF6?cluster=devnet)
- Инициализация фабрики (комиссия 1
  USD): [44fUtnEY…](https://explorer.solana.com/tx/44fUtnEYtRxxCLPXtHTwMdxyfFNNwbC5GfMHD4MGuxG824ujwWPDxohmp4f8XRqBD32J3CryCwGiE1iuB9X2Zdmt?cluster=devnet)
- Обновление цены feeder'ом: [2kuwWZNn…](https://explorer.solana.com/tx/2kuwWZNntgDcyF8waAyVmTFLwPiZ6q2aT5ipe6ygs2SmGFUUaGYCdA3xDrSyfB22WC83FAgWH6Xm4nyHmorc4sQ1?cluster=devnet)
- Обновление программ до текущей версии: oracle [2P96BG3S…](https://explorer.solana.com/tx/2P96BG3SieT1UvXNvyXeHhX3xN7fWnsUpJjJxeVVdJMXSSvWk9SZmiFRdoV3VUBhYkMohT95a6YVSbUjWnn95uk2?cluster=devnet),
  minter [64TwDAbc…](https://explorer.solana.com/tx/64TwDAbcxDjqNpvbStw4sWjTmZ19ftygpJKEXZ3smB7NeqNxVu1LAtTv8U2W3Vztvm6KAUuUqToTXj8kan8gKXnx?cluster=devnet)

## Архитектура

```
 источник цены ──► backend/feeder ──update_price──► oracle: OracleState PDA
   (мок)          (retry той же подписи,             admin, price, decimals,
                   rate limit, heartbeat,            last_updated_slot, bump
                   priority fee)
                                                            │ чтение + проверки
                                                            ▼
 пользователь ──create_token(decimals, supply)──► minter
                                                   1. проверка оракула: владелец, PDA,
                                                      decimals = 6, price > 0, свежесть
                                                   2. fee = fee_usd · 10⁹ / price  (u128, checked)
                                                   3. CPI System: payer → treasury
                                                   4. CPI SPL Token / ATA: Mint + ATA + MintTo
                                                   5. emit TokenCreated
```

### `programs/oracle`

- `initialize` — только upgrade authority программы: аккаунт `ProgramData` загрузчика
  (`seeds = [program_id]`, `seeds::program = BPFLoaderUpgradeable`) хранит upgrade
  authority, и он должен совпасть с подписантом. Так никто не перехватит роль admin
  между деплоем и инициализацией. Создаёт PDA `OracleState` (`seeds = ["oracle"]`),
  записывает admin, `decimals = 6`, слот.
- `update_price(new_price)` — только admin (`has_one` + `Signer`), `new_price > 0`,
  событие `PriceUpdated`. Sanity-фильтр скачка ±20% применяется относительно свежей
  предыдущей цены; если она протухла (или её ещё нет), фильтр пропускается — иначе после
  простоя feeder'а оракул нельзя было бы обновить, если рынок ушёл дальше 20%.
- `OracleState::require_fresh` — проверка свежести для потребителей:
  `slot − last_updated_slot ≤ 100` (слот ≈ 400 мс, это не wall-clock).

### `programs/minter`

- `initialize(fee_usd)` — только upgrade authority (так же, как в оракуле). Создаёт PDA
  `MinterConfig` (admin, treasury, fee_usd). Комиссия и казна хранятся в state:
  пользователь не может передать свою комиссию или свою «казну».
- `update_config(fee_usd)` — только admin (`has_one` + `Signer`): меняет комиссию и
  казну (новая казна передаётся аккаунтом `SystemAccount`), событие `ConfigUpdated`.
- `create_token(decimals, initial_supply)`:
    - оракул принимается как `Account<OracleState>` (владелец — программа oracle) с
      `seeds::program = oracle::ID` (это именно PDA оракула), что защищает от подмены
      оракула;
    - формат цены сверяется с `oracle::PRICE_DECIMALS` — константа не дублируется;
    - `fee_lamports` считается в `u128` с `checked_*` и проверкой обратного сужения в
      `u64`;
    - mint authority — PDA `["mint_authority", mint]`; минт подписывается через
      `invoke_signed`, пользователь не может доминтить токен в обход фабрики;
    - коды ошибок minter начинаются с 7000, чтобы не пересекаться с ошибками оракула
      (6000+), которые minter пробрасывает (`StaleOracle = 6003`).

### `backend` (off-chain)

- `init` — идемпотентная инициализация oracle (с первой ценой) и minter.
- `feeder` — мок-цена (случайное блуждание ±0.5%) → `update_price`: лимит частоты,
  пропуск неизменённой цены и heartbeat (чтобы оракул не устаревал), опциональная
  priority fee, лог signature и `last_updated_slot`.
- `create_token` — CLI для создания токена через фабрику.
- Повторы при сетевых ошибках (`retry.rs`, до 3 попыток): транзакция подписывается один
  раз, и повторно отправляется та же подпись — сеть не исполнит её дважды. Перед повтором
  проверяется статус подписи: если транзакция уже исполнилась, а ответ RPC потерялся,
  это успех, а не ошибка. Ошибки самой программы не повторяются.

## Стек

Материалы курса написаны под Anchor 0.32 / Solana 1.18; проект переведён на актуальные
версии:

|                    | Версия                       | Что изменилось относительно курса                                                                                                        |
|--------------------|------------------------------|------------------------------------------------------------------------------------------------------------------------------------------|
| Anchor             | 1.2.2                        | `CpiContext::new(program_id, …)` принимает `Pubkey`; TS-пакет `@anchor-lang/core`; сборка по умолчанию в SBPFv3; `anchor program deploy` |
| Agave (Solana CLI) | 4.3.2                        | нужен для деплоя финального SBPFv3 (на 3.x локальный валидатор отвергает такие ELF)                                                      |
| Тесты              | LiteSVM 0.15                 | in-process виртуальная машина Solana вместо `solana-test-validator` + TS                                                                 |
| Rust               | 1.89 (`rust-toolchain.toml`) |                                                                                                                                          |
| Клиент             | `anchor-client` 1.2.2        | `solana-sdk` раздроблен на отдельные крейты (`solana-keypair`, `solana-signature`, …)                                                    |

## Запуск

Требования: Rust (через rustup), [Agave](https://docs.anza.xyz/cli/install) ≥ 4.3,
Anchor CLI 1.2.2 (`avm install 1.2.2`), [just](https://github.com/casey/just).

```bash
just            # список команд
just install    # проверка инструментов, зависимости backend
just test       # все тесты без сети
```

### Полный цикл на localnet

```bash
# терминал 1
just validator

# терминал 2
just build
just deploy
cp backend/.env.example backend/.env   # ORACLE_PROGRAM_ID / MINTER_PROGRAM_ID — из `anchor keys list`
just init                              # тем же ключом, что и deploy; печатает ORACLE_STATE_PUBKEY → в backend/.env
just backend                           # feeder

# терминал 3
just create-token 6 1000               # decimals, supply в целых токенах
```

### Devnet

```bash
just build
just deploy devnet        # первый деплой или обновление тех же Program ID
just init devnet
just backend devnet
just create-token 6 1000 devnet
```

## Тесты

`just test` запускает 31 тест:

- **oracle (LiteSVM, 9)**: initialize (admin, decimals = 6, slot), повторный initialize,
  initialize не от upgrade authority, update_price от admin и не-admin, нулевая цена,
  скачок > 20%, обход фильтра скачка для протухшей цены, граница свежести.
- **minter: расчёт комиссии (unit, 6)**: типовые значения, округление вниз, нулевая
  комиссия, нулевая цена, переполнение `u64`.
- **minter (LiteSVM, 11)**: полный сценарий (комиссия в казне, decimals, supply, mint
  authority = PDA, ATA, событие), устаревший оракул (`StaleOracle`), граница свежести,
  оракул без цены, неверные decimals, подменённый оракул (`AccountOwnedByWrongProgram`),
  чужая казна, нулевой supply, initialize не от upgrade authority, update_config от
  admin (новая комиссия уходит в новую казну) и от не-admin.
- **backend (unit, 5)**: правила публикации feeder'а (лимит частоты, идемпотентность,
  heartbeat).

## Безопасность

Учебный проект, развёрнут только в Devnet и не проходил аудит. Сообщения об уязвимостях —
через [GitHub Issues](https://github.com/konarlook/blockchain-08/issues). Метаданные для
on-chain публикации — в `security.json`.

## Структура

```
programs/oracle/     on-chain оракул цены SOL/USD
programs/minter/     on-chain фабрика токенов
backend/             init, feeder, create_token (отдельный Cargo workspace)
justfile             команды localnet/devnet
security.json        контакты и политика безопасности
```