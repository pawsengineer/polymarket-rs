use super::rounding::{decimal_to_token_u64, fix_amount_rounding, ROUNDING_CONFIG};
use crate::config::get_contract_config;
use crate::error::{Error, Result};
use crate::orders::RoundConfig;
use crate::signing::{sign_order_message, EthSigner, Order};
use crate::types::{
    CreateOrderOptions, ExtraOrderArgs, MarketOrderArgs, OrderArgs, Side, SignatureType,
    SignedOrderRequest,
};
use crate::utils::get_current_unix_time_secs;
use alloy_primitives::{Address, U256};
use rand::{thread_rng, Rng};
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy::{MidpointTowardZero, ToZero};
use std::str::FromStr;

/// Generate a random seed for order salt
fn generate_seed() -> Result<u64> {
    let mut rng = thread_rng();
    let y: f64 = rng.gen();
    let timestamp = get_current_unix_time_secs()?;
    let a: f64 = timestamp as f64 * y;
    Ok(a as u64)
}

/// Builder for creating and signing orders
pub struct OrderBuilder {
    signer: Box<dyn EthSigner>,
    sig_type: SignatureType,
    funder: Address,
}

impl OrderBuilder {
    /// Create a new OrderBuilder
    ///
    /// # Arguments
    /// * `signer` - The Ethereum signer to use for signing orders
    /// * `sig_type` - The signature type (defaults to EOA if None)
    /// * `funder` - The address funding the order (defaults to signer address if None)
    pub fn new(
        signer: impl EthSigner + 'static,
        sig_type: Option<SignatureType>,
        funder: Option<Address>,
    ) -> Self {
        let sig_type = sig_type.unwrap_or(SignatureType::Eoa);
        let signer_addr = signer.address();
        let funder = funder.unwrap_or(signer_addr);

        Self {
            signer: Box::new(signer),
            sig_type,
            funder,
        }
    }

    /// Get the signature type as u8
    pub fn get_sig_type(&self) -> u8 {
        self.sig_type.to_u8()
    }

    /// Calculate order amounts for a limit order
    fn get_order_amounts(
        &self,
        side: Side,
        size: Decimal,
        price: Decimal,
        round_config: &super::rounding::RoundConfig,
    ) -> (u64, u64) {
        let raw_price = price.round_dp_with_strategy(round_config.price, MidpointTowardZero);

        match side {
            Side::Buy => {
                let raw_taker_amt = size.round_dp_with_strategy(round_config.size, ToZero);
                let raw_maker_amt = raw_taker_amt * raw_price;
                let raw_maker_amt = fix_amount_rounding(raw_maker_amt, round_config);
                (
                    decimal_to_token_u64(raw_maker_amt),
                    decimal_to_token_u64(raw_taker_amt),
                )
            }
            Side::Sell => {
                let raw_maker_amt = size.round_dp_with_strategy(round_config.size, ToZero);
                let raw_taker_amt = raw_maker_amt * raw_price;
                let raw_taker_amt = fix_amount_rounding(raw_taker_amt, round_config);

                (
                    decimal_to_token_u64(raw_maker_amt),
                    decimal_to_token_u64(raw_taker_amt),
                )
            }
        }
    }

    /// Calculate order amounts for a market order
    fn get_market_order_amounts(
        &self,
        amount: Decimal,
        price: Decimal,
        round_config: &RoundConfig,
    ) -> (u64, u64) {
        let raw_maker_amt = amount.round_dp_with_strategy(round_config.size, ToZero);
        let raw_price = price.round_dp_with_strategy(round_config.price, MidpointTowardZero);

        let raw_taker_amt = raw_maker_amt / raw_price;

        let raw_taker_amt = fix_amount_rounding(raw_taker_amt, round_config);

        (
            decimal_to_token_u64(raw_maker_amt),
            decimal_to_token_u64(raw_taker_amt),
        )
    }

    /// Create a market order
    ///
    /// Market orders are executed at the best available price by walking the order book.
    pub fn create_market_order(
        &self,
        chain_id: u64,
        order_args: &MarketOrderArgs,
        price: Decimal,
        extras: &ExtraOrderArgs,
        options: CreateOrderOptions,
    ) -> Result<SignedOrderRequest> {
        let tick_size = options
            .tick_size
            .ok_or_else(|| Error::MissingField("tick_size".to_string()))?;

        let neg_risk = options
            .neg_risk
            .ok_or_else(|| Error::MissingField("neg_risk".to_string()))?;

        let round_config = ROUNDING_CONFIG
            .get(&tick_size)
            .ok_or_else(|| Error::InvalidParameter(format!("Invalid tick_size: {}", tick_size)))?;

        let (maker_amount, taker_amount) =
            self.get_market_order_amounts(order_args.amount, price, round_config);

        let contract_config = get_contract_config(chain_id, neg_risk)?;

        let exchange_address = Address::from_str(&contract_config.exchange)
            .map_err(|e| Error::Config(format!("Invalid exchange address: {}", e)))?;

        self.build_signed_order(
            order_args.token_id.clone(),
            order_args.side,
            chain_id,
            exchange_address,
            maker_amount,
            taker_amount,
            0, // Market orders have 0 expiration
            extras,
        )
    }

    /// Create a limit order
    ///
    /// Limit orders are executed at a specific price or better.
    pub fn create_order(
        &self,
        chain_id: u64,
        order_args: &OrderArgs,
        expiration: u64,
        extras: &ExtraOrderArgs,
        options: CreateOrderOptions,
    ) -> Result<SignedOrderRequest> {
        let tick_size = options
            .tick_size
            .ok_or_else(|| Error::MissingField("tick_size".to_string()))?;

        let neg_risk = options
            .neg_risk
            .ok_or_else(|| Error::MissingField("neg_risk".to_string()))?;

        let round_config = ROUNDING_CONFIG
            .get(&tick_size)
            .ok_or_else(|| Error::InvalidParameter(format!("Invalid tick_size: {}", tick_size)))?;

        let (maker_amount, taker_amount) = self.get_order_amounts(
            order_args.side,
            order_args.size,
            order_args.price,
            round_config,
        );

        let contract_config = get_contract_config(chain_id, neg_risk)?;

        let exchange_address = Address::from_str(&contract_config.exchange)
            .map_err(|e| Error::Config(format!("Invalid exchange address: {}", e)))?;

        self.build_signed_order(
            order_args.token_id.clone(),
            order_args.side,
            chain_id,
            exchange_address,
            maker_amount,
            taker_amount,
            expiration,
            extras,
        )
    }

    /// Build and sign an order
    #[allow(clippy::too_many_arguments)]
    fn build_signed_order(
        &self,
        token_id: String,
        side: Side,
        chain_id: u64,
        exchange: Address,
        maker_amount: u64,
        taker_amount: u64,
        expiration: u64,
        extras: &ExtraOrderArgs,
    ) -> Result<SignedOrderRequest> {
        let seed = generate_seed()?;
        let taker_address = Address::from_str(&extras.taker)
            .map_err(|e| Error::InvalidParameter(format!("Invalid taker address: {}", e)))?;

        let u256_token_id = U256::from_str_radix(&token_id, 10)
            .map_err(|e| Error::InvalidParameter(format!("Invalid token_id: {}", e)))?;

        let order = Order {
            salt: U256::from(seed),
            maker: self.funder,
            signer: self.signer.address(),
            taker: taker_address,
            tokenId: u256_token_id,
            makerAmount: U256::from(maker_amount),
            takerAmount: U256::from(taker_amount),
            expiration: U256::from(expiration),
            nonce: extras.nonce,
            feeRateBps: U256::from(extras.fee_rate_bps),
            side: side.to_u8(),
            signatureType: self.sig_type.to_u8(),
        };

        let signature = sign_order_message(&self.signer, order, chain_id, exchange)?;

        Ok(SignedOrderRequest {
            salt: seed,
            maker: self.funder.to_checksum(None),
            signer: self.signer.address().to_checksum(None),
            taker: taker_address.to_checksum(None),
            token_id,
            maker_amount: maker_amount.to_string(),
            taker_amount: taker_amount.to_string(),
            expiration: expiration.to_string(),
            nonce: extras.nonce.to_string(),
            fee_rate_bps: extras.fee_rate_bps.to_string(),
            side: side.as_str().to_string(),
            signature_type: self.sig_type.to_u8(),
            signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_seed() {
        let seed1 = generate_seed().unwrap();
        let seed2 = generate_seed().unwrap();
        // Seeds should be different (very unlikely to be the same)
        assert_ne!(seed1, seed2);
    }
}
