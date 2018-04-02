### Welcome

Tactician is an AI player for the card game Dominion, using Monte Carlo Tree Search. Only the cards from the first game kingdom (Cellar, Market, Militia, Mine, Moat, Remodel, Smithy, Village, Woodcutter, Workshop) are supported.

To see Tactician play one game against a simple big money strategy, you can build and run the Tactician executable with:
    
    cargo build
    
You can also run directly from cargo with:

    cargo run --release -- 1

### Ideas for Improving Play

* Move-average sampling technique: "Maintain average reward statistics for each action independently of where it occurs in the game tree, and use these statistics to bias the simulation policy." [https://pdfs.semanticscholar.org/d10e/31ed85cc6ea79d3d961730da2b07c32aa984.pdf

* Instead of playing each simulation to the game's end with random play, limit the number of moves/actions evaluated, and evaluate the result with a simple deck hueristic. [https://www.sciencedirect.com/science/article/pii/S0004370210000536]

* Better capture information from simulation results .[http://orangehelicopter.com/academic/papers/ai_icarus.pdf]
