use mimir_crypto::secp256k1::Address;
use project::node::Node;
use project::actor::Actor;
use project::contract::Contract;
use types::Tags;


// TODO: Implement `InternalAccount`, or similar, to represent an account
// with known values for `secret`, `password`, etc...
//
// NOTE: The internal/external distinction used for actors & nodes does not map
// to the general account level perfectly since contracts are "internal" but don't
// hold most of the properties one would associate with an internal entitiy...


#[derive(Debug,Copy,Clone)]
pub enum Account<'a> {
    Node(Node<'a>),
    Actor(Actor<'a>),
    Contract(&'a Contract),
}


impl<'a> Account<'a> { 

    pub fn name(&self) -> &'a str {
        match self {
            Account::Node(entity) => entity.name(),
            Account::Actor(entity) => entity.name(),
            Account::Contract(entity) => &entity.name,
        }
    }

    pub fn address(&self) -> Address {
        match self {
            Account::Node(entity) => entity.account_addr(),
            Account::Actor(entity) => entity.address(),
            Account::Contract(entity) => entity.addr,
        }
    }

    pub fn balance(&self) -> u64 { // TODO: add balance configuration for nodes & contracts
        match self {
            Account::Node(_) => 1,
            Account::Actor(entity) => entity.balance(),
            Account::Contract(_) => 1,
        }
    }

    pub fn tags(&self) -> &'a Tags {
        match self {
            Account::Node(entity) => entity.tags(),
            Account::Actor(entity) => entity.tags(),
            Account::Contract(entity) => &entity.tags,
        }
    }

    pub fn node(&self) -> Option<Node<'a>> {
        match self {
            Account::Node(entity) => Some(*entity),
            _other => None
        }
    }

    pub fn actor(&self) -> Option<Actor<'a>> {
        match self {
            Account::Actor(entity) => Some(*entity),
            _other => None,
        }
    }

    pub fn contract(&self) -> Option<&'a Contract> {
        match self {
            Account::Contract(entity) => Some(*entity),
            _other => None,
        }
    }
}


impl<'a> From<Node<'a>> for Account<'a> {

    fn from(entity: Node<'a>) -> Self {
        Account::Node(entity)
    }
}


impl<'a> From<Actor<'a>> for Account<'a> {

    fn from(entity: Actor<'a>) -> Self {
        Account::Actor(entity)
    }
}


impl<'a> From<&'a Contract> for Account<'a> {

    fn from(entity: &'a Contract) -> Self {
        Account::Contract(entity)
    }
}

