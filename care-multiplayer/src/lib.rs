//! Easy client/server or p2p multiplayer functionality
//! If you want to implement a basic client using a default game server, or
//! using automatic p2p, you should only need the client module.
//!
//! # Trust
//! An important decision you're going to have to make when writing multiplayer
//! code is the trust model. You have three major options:
//!
//! ## Trust the server
//! Trusting the server is very common in real games, the idea is that the
//! server holds the "source of truth" and all clients rely on that server
//! state and simply try to maintain their state as a copy of the server's
//! state. [Minecraft](https://minecraft.net/) uses this approach.
//!
//! To use this technique using this library, you'll want to use the server
//! module to create a custom server that contains your game's core logic,
//! and create a client that resets any variables that may be visible over
//! the network to the values that are on the server. The [`client::SyncedValue`]
//! and [`server::SyncedValue`] types might be helpful for this.
//!
//! # Trust everyone
//! Trusting everyone means that each client handles it's own calculations
//! and all clients "trust" that all other clients are reporting correct
//! information. This requires assigning the responsibility to simulate
//! any shared components to one of the clients. [VRChat](https://vrchat.com/)
//! uses this approach.
//!
//! # Trust noone
//! Trusting noone means that each client simulates the *entire* game. This
//! requires that all of the calculations are perfect and identical for all
//! copies of the game, including random elements. It also requires a way of
//! resolving situations where multiple clients cannot come to an agreement
//! over the state of the game. [Dolphin](https://dolphin-emu.org/)'s netplay
//! feature uses this approach.

pub mod server;
pub mod client;

