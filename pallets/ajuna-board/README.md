# Ajuna Network Pallet Board

The Board Game pallet provides an implementation of a turn based board game. The game logic for the board game is implemented with the trait `TurnBasedGame` and provided as part of the configuration of this pallet.  Extrinsics are used in the creation and playing of a game with state being temporarily stored on chain.  The creation and playing of the game are delegated, after being gated by the pallet, to the core game logic.

## Purpose
Validation and state management of board games

## Dependencies
`ajuna-common` crate

### Traits
In order to use the pallet an implementation of `TurnBasedGame` would need to provided in the configuration.

## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
# external pallets
pallet-ajuna-board = { default-features = false, git = "https://github.com/ajuna-network/Ajuna" }
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    'pallet-ajuna-board/std',
]
```

### Runtime `lib.rs`

You should implement its trait with something like:

```rust
impl pallet_ajuna_board::Config for Test {
	type Event = Event;
	type MaxNumberOfPlayers = MaxNumberOfPlayers;
	type BoardId = u32;
	type PlayersTurn = u32;
	type GameState = GameState;
	type Game = Game;
}
```

and include it in your `construct_runtime!` macro:

```rust
```

### Genesis Configuration

This `ajuna-board` pallet does not have any genesis configuration.

### Types

No additional types

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
