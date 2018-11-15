use mimir_crypto::secp256k1::{Address,Secret,Signer};
use types::{Tags,Error};
use util;
use rand;


#[derive(Debug,Clone)]
pub struct InternalActor {
    actor_name: String,
    signer: Signer, 
    actor_pass: String,
    balance: u64,
    tags: Tags,
}


impl InternalActor {

    pub fn try_from(config: InternalActorConfig) -> Result<Self,Error> {
        let signer = config.signer()?;
        let InternalActorConfig { actor_name, actor_pass, balance, tags, .. } = config;
        Ok(Self { actor_name, signer, actor_pass, balance, tags })
    }

    pub fn name(&self) -> &str { &self.actor_name }

    pub fn secret(&self) -> Secret { self.signer.secret() }

    pub fn address(&self) -> Address { self.signer.address() }

    pub fn password(&self) -> &str { &self.actor_pass }
}


#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case",deny_unknown_fields)]
pub struct InternalActorConfig {
    actor_name: String, 
    #[serde(default = "rand::random")]
    actor_secret: Secret,
    #[serde(default = "util::rand_pass")]
    actor_pass: String,
    #[serde(default)]
    balance: u64,
    #[serde(default)]
    tags: Tags,
}


impl InternalActorConfig {

    pub fn new(actor_name: String) -> Self { 
        let actor_secret = rand::random();
        let actor_pass = util::rand_pass();
        let balance = 1;
        let tags = Default::default();
        Self { actor_name, actor_secret, actor_pass, balance, tags }
    }

    pub fn signer(&self) -> Result<Signer,Error> {
        let signer = Signer::new(&self.actor_secret)?;
        Ok(signer)
    }
}


pub type ExternalActorConfig = ExternalActor;


#[derive(Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case",deny_unknown_fields)]
pub struct ExternalActor {
    actor_name: String,
    address: Address,
    #[serde(default)]
    balance: u64,
    #[serde(default)]
    tags: Tags,
}


#[derive(Debug,Clone)]
pub struct Actors {
    internal: Vec<InternalActor>,
    external: Vec<ExternalActor>,
}


impl Actors {

    pub fn try_from(config: ActorConfigs) -> Result<Self,Error> {
        let ActorConfigs { internal, external } = config;
        let internal = internal.into_iter()
            .map(InternalActor::try_from)
            .collect::<Result<_,_>>()?;
        Ok(Self { internal, external })
    }

    pub fn iter(&self) -> impl Iterator<Item=Actor> {
        self.internal.iter().map(Actor::from).chain(
            self.external.iter().map(Actor::from)
        )
    }
}


#[derive(Default,Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "kebab-case",deny_unknown_fields)]
pub struct ActorConfigs {
    #[serde(default,skip_serializing_if = "Vec::is_empty")]
    internal: Vec<InternalActorConfig>,
    #[serde(default,skip_serializing_if = "Vec::is_empty")]
    external: Vec<ExternalActorConfig>,
}


impl ActorConfigs {

    pub fn insert(&mut self, actor: impl Into<ActorConfig>) {
        match actor.into() {
            ActorConfig::Internal(actor) => self.internal.push(actor),
            ActorConfig::External(actor) => self.external.push(actor),
        }
    }

    pub fn import(&mut self, other: Self) {
        let Self { internal, external } = other;
        self.internal.extend(internal);
        self.external.extend(external);
    }

    pub fn is_empty(&self) -> bool {
        self.internal.is_empty() && self.external.is_empty()
    }
}


pub enum ActorConfig {
    Internal(InternalActorConfig),
    External(ExternalActorConfig),
}


impl From<InternalActorConfig> for ActorConfig {

    fn from(config: InternalActorConfig) -> Self { ActorConfig::Internal(config) }
}


impl From<ExternalActorConfig> for ActorConfig {

    fn from(config: ExternalActorConfig) -> Self { ActorConfig::External(config) }
}


#[derive(Debug,Copy,Clone)]
pub enum Actor<'a> {
    Internal(&'a InternalActor),
    External(&'a ExternalActor),
}


impl<'a> Actor<'a> {

    pub fn name(&self) -> &'a str {
        match self {
            Actor::Internal(actor) => &actor.actor_name,
            Actor::External(actor) => &actor.actor_name,
        }
    }

    pub fn address(&self) -> Address {
        match self {
            Actor::Internal(actor) => actor.signer.address(),
            Actor::External(actor) => actor.address,
        }
    }

    pub fn balance(&self) -> u64 {
        match self {
            Actor::Internal(actor) => actor.balance,
            Actor::External(actor) => actor.balance,
        }
    }

    pub fn tags(&self) -> &'a Tags {
         match self {
            Actor::Internal(actor) => &actor.tags,
            Actor::External(actor) => &actor.tags,
        }
    }

    pub fn internal(&self) -> Option<&'a InternalActor> {
        if let Actor::Internal(actor) = self {
            Some(actor)
        } else {
            None
        }
    }

    pub fn external(&self) -> Option<&'a ExternalActor> {
        if let Actor::External(actor) = self {
            Some(actor)
        } else {
            None
        }
    }
}


impl<'a> From<&'a InternalActor> for Actor<'a> {

    fn from(actor: &'a InternalActor) -> Self {
        Actor::Internal(actor)
    }
}


impl<'a> From<&'a ExternalActor> for Actor<'a> {

    fn from(actor: &'a ExternalActor) -> Self {
        Actor::External(actor)
    }
}



