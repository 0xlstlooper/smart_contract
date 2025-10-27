# Defi BonBon

Problems we solve:
1. **Yield cannot be leveraged** - I can long 10x SOL, but I just can't leverage my yield.
2. **Yield aggregators are always centralised** - You trust an operator and give up a fee for that privilege.

With a **novel and unique defi primitive**, we solve them both at the same time (because they're two sides of the same coin actually).





## How it works? - high level POV

You deposit your idle capital in a vault (there is 1 vault per "distinct asset" (so 1 for $, 1 for sol, 1 for €, etc...)).  
Any **yield farmor** can come, deposit a defi position delta-neutral to that the underlying asset of the vault (eg. deposit JitoSol in the Sol vault, not USDC in the Sol vault, and for now only tokens not entire defi position).  
All the yield farmors do an auction on how much fixed apy they are willing to pay to the vault.  
The yield farmors that won this auction then borrow assets from that vault to leverage their defi position and pay that fixed yield - a bit akin to a multiply position in lending markets, but safer due to the fixed apy.  

#### Why participants will want to use the platform?

The yield famor pocket the difference in yield between its defi position, and the fixed APY he needs to pay to the vault.

You, who just deposited your idle capital, simply collect this APY without any research needed, and the whole thing acts as a decentralised yield aggregator because the most performant defi position will win the auction and the vault assets will be deployed to these positions.

#### Lockups?

Anyone can deposit/withdraw at anytime (some costs are incurred on deposit/withdraw due to having to do some swaps, akin to a multiply deposit/withdraw essentially, but costs will be less we think).

#### Market forces.

Market forces will give competitive yield to both participants, the yield farmors' margins will be razor thin due to the leverage incur, if your defi position earns 7% you can pay 6.5%, and earn 0.5% on 10x your already existing defi position for essentially 0 negative apy risk due to the fact that the APY you pay is fixed not variable.

#### Excepted behaviors at equilibrum state

At equilibrium, we except that yield farmors will want an additionnal APY of only a few %, as close to none other options on the market exists to earn additionnal risk-free yield on top of a yield bearing asset. Few %, let's say 3, at 10x leverage, is only a 0.3% difference. Let's round it to 0.5% to compensate also for their increases loses in case of a depeg event.  
Therefore, we except that the yield given by this decentralised aggregator will be the absolute best yield available minus 0.5%, which if the vault earn 10% APY is actually a performance fee of only 5% way below all curated vaults (when at equilibrum, of course now they charge 0% rate), and with actually more "skin in the game" from the curators (here the yield farmors) because if they wish to deploy the vault's money into a risky asset they have to place their own money into that risky asset and loses a lot with small depegs due to leverage.

At equilibrum, we also except that the orderbook composition won't change so much, and as such as a yield farmors it'll be pretty predictable to leverage your yield and earn that additionnal %. Yields of yield bearing assets doesn't change everyday, and even if they do, that's for pretty risky assets which will have a cap on the total composition they can take on the pool. As such, the pool will always be composed of some less risky assets, with stable yields, and being a competitive offer on that asset will be straightforward: the average yield of it minus 0.3%.

All in all, at equilibrum, we except the whole structure to be quite stable, require close to no supervision from both lenders and yield farmors, and deliviring them good and stable yield in a true win-win situation that doesn't exists in any other defi product accross any chain.




## How it works - technical POV

todo

### Flow of use of the functions of the contract

Flow of use of lender deposit funds (function `deposit`):  

The frontend of the lender checks which token**S** and amounts of them he is supposed to deposit.  
Then, with a single call to this function he deposits all the tokens at once.  
On deposit, the user must therefore swap its assets to the required assets. He is required to do swaps of 1x the volume he deposited.

Funny enough, because the frontend do the swap separately, if the user gets frontrunned, the transaction will fail because the required tokens to be depositted will be different.  
--> No slippage parameter for this code, if you get frontrunned, you need to do a new tx.  
(Although the frontend has a slippage parameter when doing the swaps to get the required tokens, but we use exact output, so the user will always get the exact amount of tokens needed for the deposit, even if what he pays will differ because of slippage)

Flow of use of the rest: same. All the functions are sensible to frontrunning, users dont lose money, they just have txs that fail.
Volume of swaps required by function called:
+ `Deposit`/`Withdraw`: `1 x amount`.
+ `Place bid`/`Remove bid`:
    - Either, the required split is the same, and in this case we do no swaps. In this case, `0`.
    - Or, the required split changes, and we need to do `leverage x amount`. Eg, if the split before was 1yield farmor that deposited 1JupSol such that 10Sol of lenders are split into 10JupSol. Imagine we deposit 1JitoSol such that our offer is more competitive than existing offers, then the 10JupSol must be swapped to 10JitoSol. So the volume of swapped required is `levarage x amount`.

--> One of the known issue that'll need be addressed in the mainnet version, is that ots a bit toxic, that if JitoSol becomes more competitive, and someone do the swap of JupSol to JitoSol, then someone else can then be a more competitive JitoSol offer wihtout doing any swaps, and as such we didn't rewarded who did the swap. Probably the solution is to award whoever did the swap a bonus APY for a given time in order to offset him the cost of the swap. It's still unclear whats the best "game theory" solution. We didn't implement anything related to that in the code, for now whoever did the swap eat the cost.




## How to test/run it

### The smart contract

Build: `anchor build`.

Test: `anchor test`.

### The isolated orderbook rust datastructure

In file `state/all_assets.rs`.

Run its isolated tests: `cargo test`.

### Run the website:

`http://github.com/0xlstlooper/vacances-a-bali`