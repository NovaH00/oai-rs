//! Core types and client for the OpenAI-compatible chat completion API.
//!
//! This module provides the request/response models ([`request`], [`response`]),
//! generation parameters ([`chat_config`]), and the [`LLMClient`] that ties them
//! together.

mod chat_config;
mod client;
mod request;
mod response;

pub use chat_config::ChatConfig;
pub use client::LLMClient;
