use crate::*;

/// approval callbacks from NFT Contracts

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MarketArgs {
    pub market_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<U128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ft_token_id: Option<AccountId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer_id: Option<AccountId>, // offer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<U64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<U64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_auction: Option<bool>,
}

trait NonFungibleTokenApprovalsReceiver {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}

#[near_bindgen]
impl NonFungibleTokenApprovalsReceiver for Contract {
    fn nft_on_approve(
        &mut self,
        token_id: TokenId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) {
        // enforce cross contract call and owner_id is signer

        let nft_contract_id = env::predecessor_account_id();
        let signer_id = env::signer_account_id();
        assert_ne!(
            env::current_account_id(), nft_contract_id,
            "Error: nft_on_approve should only be called via cross-contract call"
        );
        assert_eq!(owner_id, signer_id, "Error: owner_id should be signer_id");

        assert!(
            self.approved_nft_contract_ids.contains(&nft_contract_id),
            "Error: nft_contract_id is not approved"
        );

        let MarketArgs {
            market_type,
            price,
            ft_token_id,
            buyer_id,
            started_at,
            ended_at,
            is_auction
        } = near_sdk::serde_json::from_str(&msg).expect("Not valid MarketArgs");

        if market_type == "sale" {
            assert!(price.is_some(), "Error: price not specified");

            self.internal_delete_market_data(&nft_contract_id, &token_id);

            let storage_amount = self.storage_minimum_balance().0;
            let owner_paid_storage = self.storage_deposits.get(&signer_id).unwrap_or(0);
            let signer_storage_required =
                (self.get_supply_by_owner_id(signer_id).0 + 1) as u128 * storage_amount;

            assert!(
                owner_paid_storage >= signer_storage_required,
                "Insufficient storage paid: {}, for {} sales at {} rate of per sale",
                owner_paid_storage,
                signer_storage_required / storage_amount,
                storage_amount,
            );

            let ft_token_id_res = ft_token_id.unwrap_or(near_account());

            if self.approved_ft_token_ids.contains(&ft_token_id_res) != true {
                env::panic_str(&"Error: ft_token_id not approved");
            }

            self.internal_add_market_data(
                owner_id,
                approval_id,
                nft_contract_id,
                token_id,
                ft_token_id_res,
                price.unwrap(),
                started_at,
                ended_at,
                is_auction,
            );
        } else if market_type == "accept_offer" {
            assert!(buyer_id.is_some(), "Error: Account id is not specified");
            assert!(price.is_some(), "Error: Price is not specified (for check)");

            self.internal_accept_offer(
                nft_contract_id,
                buyer_id.unwrap(),
                token_id,
                owner_id,
                approval_id,
                price.unwrap().0,
            );
        }
    }
}
