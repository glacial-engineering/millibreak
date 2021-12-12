# MILLIONSY Vulnerability Report

This document explains several vulnerabilities in the MILLIONSY program at address `5d15XQp2jYPxeQtkCWoYu84zWbbFiSoHsJK39KwH2jrf` on mainnet-beta of the Solana chain.

The code alongside this document allows for reproducing the discussed vulnerabilities. 

## Background

The MILLIONSY project's website is located at https://www.millionsy.io/.  In this report we'll be discussing the "Lottery" product of the MILLIONSY project.

The lottery being run is a pick-6 type lottery with varying payouts for matching different numbers.  Documentation is available outlining the details about how a lottery is run. The lottery is documented as verifiably random.

The lottery is run for 24hr starting daily at 1130 UTC and ending 23.5 hours later.

## Execution

The lottery is managed by a script being run by who we assume is the MILLIONSY project team which uses address `DopvZbavQEghVDkU5NeFNdZ8pYCpQsWWmwJarATYkjoe`.  Every day this executes instructions against the main program performing the following operations:

|Op|Description|Sample tx|
|--|--|--|
|`0x00`|Create game|[`3jH9....ZaYL`](https://solscan.io/tx/3jH9cg9xqoTme86z2SGA6MaxjkBZkcvDcvEN9PA1ppFNKECVFDWciu6zjy2VgRBCVcCF3PYdgR9hwQsCxuJtZaYL)|
|`0x02`|Generate winning numbers|[`2SjG....5z8k`](https://solscan.io/tx/2SjG89DEg2yn3nkWkh6rfvhR4J4TqZYvehocBKXAc5RvMb4W7ZsFicqviBGGUTpfxFYnX8eH3er3CtVaMCbB5z8k)|
|`0x03`|Set winning ticket count|[`349v....CYNj`](https://solscan.io/tx/349vWqF2xXUaBcWMidqfq9SoVmCHL1PiGpfoPuE3B61rh2cYeiFhPWi15fPRqBCXYVu36ni3VPa2sA2Ch2F1CYNj)|
|`0x04`|Pay winner|[`rbsZ....gxS6`](https://solscan.io/tx/rbsZUc7cBNCkcW7uAiifC6v912wXfsb9VHkj2jfdxpgdnxQxmfJuRde5WXsAgXoicHnjJgjJ32HHd8QFDRygxS6)|
|`0x05`|Close game|[`4yo9....mqczJ`](https://solscan.io/tx/4yo9M7ECp5HuQHBX2swmjLFzK1PzQYCnASqKmC7CwuLKyUcTEWo96xg2ymBmQ33Ts122bKYwtQuZ414iSE6mqczJ)|
|`0x06`|Update price of tickets for a game|[`5iPA....QsVq`](https://solscan.io/tx/5iPAcjF4771CzGaWTncJztzRP4iGpjeNgbLF2pDUSprfv2aogzdmHgAeiycR1N27Fdua1NUf2QUA1XWBacYuQsVq)|

The script executes instructions `02`, `03`, `04` (if needed), and `05` in quick succession daily at approx 1100:30.  The assumption is that the script is generating the random numbers, calculating the winning tix (we assume here it cached them from before the numbers were picked, more on that later), then executing `03` and `04` based on winning ticket data.

Users interact with the program using the website which issues the following operations:
|Op|Description|
|--|--|
|`0x01`|Buy ticket|

Due to the fact that the execution of the lottery is running off-chain, we have to make assumptions about its behavior. There may be more additional exploits depending on how the chain data is being read/cached, and the timing thereof.

## Fairness

### Randomness

The execution of operation `0x02` takes 6 `u64` which act as a seed to the generation of the winning numbers. These seeds can be any numbers - so as such are not proofs. **I believe that the chain's timestamp during operation execution is the other input to this RNG.**

The source of the 6 numbers being passed in is unknown, they could perhaps be some verifiable information.  This does not align with [the documentation](https://docs.millionsy.io/lottery/the-millionsy-vrf) which states:

> The MILLIONSY Lottery smart contract will only accept the random number input if it has a valid cryptographic proof, and the cryptographic proof can only be generated if the VRF process is tamper-proof.

> By letting any user having the ability to independently audit the integrity of the RNG to verify that it's unbiased, unpredictable, and manipulation, MILLIONSY wants to send a strong statement that the important role of fairness and transparency to our random number draws is undeniable.

Given that we cannot determine where the seeds in `0x02` come from, **the winning numbers of the mainnet program become effectively unexploitable by us**, but potentially exploitable if those numbers are known in advance - IF timestamp is the other input to the seed. Solana timestamp is in seconds, and the predictable schedule of the program makes this exploitable.

Note: I believe that timestamp is the other input to this seed because I'm not aware of being able to access a blockhash from a program without taking the sysvar recent blockhash account.  Clock on the other hand, can by loaded without being taken as an account.

Verdict: Concerning; need more info. Possibly acceptable risk alongside centralization risk.

### Centralization

This lottery is dependent on a centralized script, owned by the project team, which must execute offchain and issue the necessary isntructions. This could be modified to benefit others, or simply be turned off and not run.

Verdict: Centralized; closed source.

## Onchain Vulnerabilities

---

### 1 - Re-direct ticket payment

#### Problem

The `0x01` op code used to buy tickets takes the account to pay as the 4th account to its input.  **This account can be any account including the account buying the ticket.** 

#### Exploit 

Major: This effectively reduces the cost to purchase a ticket to the cost of rent (0.001392 SOL) if the account buying the ticket pays themselves for it. It also means the payout pool does not receive any funds for malicious ticket purchases.

#### Fix

Assert that the account being paid for the ticket matches the owner specified on the Game account.

---

### 2 - Re-usable ticket accounts

#### Problem

The `0x01` op code used to buy tickets takes the ticket account to use for recording the ticket data as the 2nd account to its input.  **This account is not required to be empty.** 

#### Exploit 

Major: Combined with vulnerability 1, the attacker can acquire free tickets by re-using any previously created ticket account (for either the current game or past games) and redirecting payment to themselves. Because the account already existed, the attacker does not even need to pay the rent for the ticket. This has the added impact of allowing the attacker to erase other player's tickets and claim them as the attacker's.

#### Fix

Assert that the account being passed in for the ticket has not been previously written (check a status field).

---

### 3 - Anyone update game winning ticket count

#### Problem

The `0x03` op code used to update a game with the number of winning tickets can be called by anyone because the signiture of account #2 is not checked.

#### Exploit 

UI/Minor: Updating the winning ticket count of past **or current** games could confuse users.

Moderate: If a malicious actor wins the lottery (fairly) AND other user(s) win the lottery in the same game, the malicious actor could update the # winning tickets to `1` overwriting the higher number written by the script.  Because payout calculations use this number to split the pot, a higher payout (no split pot) would be received.  *Note: there would be only a 2-4 second window to squeeze this transaction in between the script's `03` and `04`.*

#### Fix

Assert that account #2 passed to this instruction is signed.

---

### 4 - Anyone create games

#### Problem

The `0x00` op code used to create games does not check the signiture for account #2, which is the owner of the game

#### Exploit 

UI/Minor: Anyone can create games for any round and owner, likely confusing the UI and/or the script

Major (Potentially): **Depending on how the script is reading and interpreting program accounts**, it is possible through a variety of ways to mock up games both in advance, and during the 3 second window between reveal and pay.

#### Fix

Assert that account #2 passed to this instruction is signed.  Additionally, be sure that the script is only looking at games owned by itself.

---

## Conclusion

The off-chain way that this lottery is run does afford it the benefit of not having an onchain vault that is able to be withdrawn from.  The only way that funds can be paid out is by the script's execution of instructions. However, it is possible that data can be mocked up in such a ways as to confuse the script and trigger an incorrect payout.

Randomness and fairness is unknown. **If we are to trust the project team in a centralized type manner**, then the way the game runs is less of a concern - assuming the `02` operation seeds are random.

