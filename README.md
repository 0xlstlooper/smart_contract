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

### Whats "interesting"/"unusual" about it

**There is no slippage parameters**: Something interesting about the app is what does happen when you get frontrunned?
Let's say you want to deposit 100SOL, you are required to deposit 1000JitoSol, but if someone else deposit before you
maybe you now need to deposit 1000JupSol. Because we do the swaps outside the smart contract, the transaction will fail and will
need to be redone.

### Known issues

Quand un looper swap un asset pour un autre, c’est un peu chiant car il doit payer le cout, et après un autre mec peut venir supply de cet asset et passer devant -> Pour l’instant on laisse comme ça, todo voir en détail, et quand on liste un nouvel asset on peut faire le dépôt initial et payer le cout nous


### What's unclear to me

How do we receive tokens we dont have an account for, yet. Do we have to pay for the ret? Can i receive it, swap them, and close them in a single tx?


## How to test/run it

### The smart contract

Build: `anchor build`.

Test: `anchor test`.

### The isolated orderbook rust datastructure

In file `state/all_assets.rs`.

Run its isolated tests: `cargo test`.

### Run the website:

`http://github.com/0xlstlooper/vacances-a-bali`