#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype,
    Address, Env, String, Vec, panic_with_error,
};

// ─────────────────────────────────────────────
//  ERRORS & TIERS
// ─────────────────────────────────────────────
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized    = 1,
    AlreadyInitialized= 2,
    Unauthorized      = 3,
    InsufficientFunds = 4,
    InvalidCase       = 5,
    InvalidTier       = 6,
    Wait24Hours       = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tier { Common, Uncommon, Rare, Epic, Legendary, Mythic }

// ─────────────────────────────────────────────
//  DATA TYPES
// ─────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug)]
pub struct Item { pub id: u32, pub name: String, pub tier: Tier }

#[contracttype]
#[derive(Clone, Debug)]
pub struct CaseType { pub id: u32, pub name: String, pub price: u64 }

#[contracttype]
#[derive(Clone, Debug)]
pub struct InventoryEntry { pub item_id: u32, pub quantity: u32 }

#[contracttype]
pub enum DataKey {
    Admin, Initialized, ItemCount, Item(u32), CaseCount, CaseType(u32),
    Balance(Address), Inventory(Address, u32), InvKeys(Address),
    LastLogin(Address), Streak(Address),
}

// ─────────────────────────────────────────────
//  CONTRACT
// ─────────────────────────────────────────────
#[contract]
pub struct GachaContract;

#[contractimpl]
impl GachaContract {
    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::ItemCount, &0u32);
        env.storage().instance().set(&DataKey::CaseCount, &0u32);
        Self::_seed_items(&env);
        Self::_seed_cases(&env);
    }

    pub fn claim_daily(env: Env, caller: Address) -> u64 {
        caller.require_auth();
        Self::_require_initialized(&env);

        let current_time = env.ledger().timestamp();
        let last_time: u64 = env.storage().persistent().get(&DataKey::LastLogin(caller.clone())).unwrap_or(0);
        let mut streak: u32 = env.storage().persistent().get(&DataKey::Streak(caller.clone())).unwrap_or(0);

        if last_time != 0 && current_time < last_time + 86_400 {
            panic_with_error!(&env, Error::Wait24Hours);
        }

        if last_time != 0 && current_time > last_time + 172_800 { streak = 0; }
        streak += 1;
        
        let reward: u64 = if streak >= 7 { streak = 0; 500 } else { 100 };

        env.storage().persistent().set(&DataKey::LastLogin(caller.clone()), &current_time);
        env.storage().persistent().set(&DataKey::Streak(caller.clone()), &streak);
        
        let bal = Self::_get_balance(&env, &caller);
        env.storage().persistent().set(&DataKey::Balance(caller.clone()), &(bal + reward));

        reward
    }

    pub fn mint_coins(env: Env, caller: Address, recipient: Address, amount: u64) {
        caller.require_auth();
        Self::_require_admin(&env, &caller);
        let bal = Self::_get_balance(&env, &recipient);
        env.storage().persistent().set(&DataKey::Balance(recipient), &(bal + amount));
    }

    pub fn open_case(env: Env, caller: Address, case_id: u32) -> u32 {
        caller.require_auth();
        Self::_require_initialized(&env);

        let ct: CaseType = env.storage().persistent().get(&DataKey::CaseType(case_id)).unwrap_or_else(|| panic_with_error!(&env, Error::InvalidCase));
        let bal = Self::_get_balance(&env, &caller);
        
        if bal < ct.price { panic_with_error!(&env, Error::InsufficientFunds); }
        env.storage().persistent().set(&DataKey::Balance(caller.clone()), &(bal - ct.price));

        let roll = Self::_roll(&env);
        let tier = Self::_tier_from_roll(roll);
        let item_id = Self::_pick_item_in_tier(&env, &tier);
        Self::_give_item(&env, &caller, item_id);

        item_id
    }

    pub fn sell_item(env: Env, caller: Address, item_id: u32) -> u64 {
        caller.require_auth();
        let inv_key = DataKey::Inventory(caller.clone(), item_id);
        let qty: u32 = env.storage().persistent().get(&inv_key).unwrap_or(0);
        
        if qty == 0 { panic_with_error!(&env, Error::Unauthorized); }

        let item: Item = env.storage().persistent().get(&DataKey::Item(item_id)).unwrap();
        let price: u64 = match item.tier {
            Tier::Common => 10, Tier::Uncommon => 40, Tier::Rare => 120,
            Tier::Epic => 350, Tier::Legendary => 1000, Tier::Mythic => 5000,
        };

        env.storage().persistent().set(&inv_key, &(qty - 1));
        let bal = Self::_get_balance(&env, &caller);
        env.storage().persistent().set(&DataKey::Balance(caller.clone()), &(bal + price));

        price
    }

    pub fn get_balance(env: Env, user: Address) -> u64 { Self::_get_balance(&env, &user) }
    
    pub fn get_inventory(env: Env, user: Address) -> Vec<InventoryEntry> {
        let keys: Vec<u32> = env.storage().persistent().get(&DataKey::InvKeys(user.clone())).unwrap_or_else(|| Vec::new(&env));
        let mut result = Vec::new(&env);
        for id in keys.iter() {
            let qty: u32 = env.storage().persistent().get(&DataKey::Inventory(user.clone(), id)).unwrap_or(0);
            if qty > 0 { result.push_back(InventoryEntry { item_id: id, quantity: qty }); }
        }
        result
    }

    // ─────────────────────────────────────────
    //  PRIVATE HELPERS & SEEDS
    // ─────────────────────────────────────────
    fn _require_initialized(env: &Env) { if !env.storage().instance().has(&DataKey::Initialized) { panic_with_error!(env, Error::NotInitialized); } }
    fn _require_admin(env: &Env, caller: &Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap_or_else(|| panic_with_error!(env, Error::NotInitialized));
        if *caller != admin { panic_with_error!(env, Error::Unauthorized); }
    }
    fn _get_balance(env: &Env, user: &Address) -> u64 { env.storage().persistent().get(&DataKey::Balance(user.clone())).unwrap_or(0u64) }
    fn _roll(env: &Env) -> u32 {
        let ts = env.ledger().timestamp();
        let seq = env.ledger().sequence();
        let hash = (ts ^ (seq as u64).wrapping_mul(2_654_435_761)) as u32;
        hash % 10_000
    }
    fn _tier_from_roll(roll: u32) -> Tier {
        match roll {
            0..=5999 => Tier::Common,
            6000..=8499 => Tier::Uncommon,
            8500..=9499 => Tier::Rare,
            9500..=9849 => Tier::Epic,
            9850..=9899 => Tier::Legendary,
            _ => Tier::Mythic, // 1% chance (9900-9999)
        }
    }
    fn _pick_item_in_tier(env: &Env, tier: &Tier) -> u32 {
        let count: u32 = env.storage().instance().get(&DataKey::ItemCount).unwrap_or(0);
        let mut candidates: Vec<u32> = Vec::new(env);
        for id in 1..=count {
            if let Some(item) = env.storage().persistent().get::<DataKey, Item>(&DataKey::Item(id)) {
                if &item.tier == tier { candidates.push_back(id); }
            }
        }
        if candidates.is_empty() { return 1; } // Fallback to item 1
        let ts = env.ledger().timestamp();
        let idx = (ts as u32) % (candidates.len() as u32);
        candidates.get(idx).unwrap()
    }
    fn _give_item(env: &Env, user: &Address, item_id: u32) {
        let key = DataKey::Inventory(user.clone(), item_id);
        let qty: u32 = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &(qty + 1));
        let inv_key = DataKey::InvKeys(user.clone());
        let mut keys: Vec<u32> = env.storage().persistent().get(&inv_key).unwrap_or_else(|| Vec::new(env));
        if !keys.contains(&item_id) { keys.push_back(item_id); env.storage().persistent().set(&inv_key, &keys); }
    }

    fn _seed_items(env: &Env) {
        let items: [(&str, Tier); 7] = [
            ("Cyber Scraps", Tier::Common),
            ("Neon Cable", Tier::Uncommon),
            ("Glitch Visor", Tier::Rare),
            ("Holo-Deck", Tier::Epic),
            ("Zero-Day Exploit", Tier::Legendary),
            ("Source Code", Tier::Mythic),
            ("God Particle", Tier::Mythic),
        ];
        let mut id = 0u32;
        for (name, tier) in &items {
            id += 1;
            let item = Item { id, name: String::from_str(env, name), tier: tier.clone() };
            env.storage().persistent().set(&DataKey::Item(id), &item);
        }
        env.storage().instance().set(&DataKey::ItemCount, &id);
    }

    fn _seed_cases(env: &Env) {
        let cases: [(&str, u64); 5] = [
            ("Starter Case", 50), ("Warrior Cache", 150),
            ("Mystic Chest", 300), ("Divine Vault", 600), ("Chaos Box", 999),
        ];
        let mut id = 0u32;
        for (name, price) in &cases {
            id += 1;
            let ct = CaseType { id, name: String::from_str(env, name), price: *price };
            env.storage().persistent().set(&DataKey::CaseType(id), &ct);
        }
        env.storage().instance().set(&DataKey::CaseCount, &id);
    }
}