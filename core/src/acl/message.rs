use alloc::collections::btree_set::BTreeSet;
use alloc::string::String;

type Aid = String;

type Encoding = String;

type Ontology = String;

type Content = String;

type Protocol = String;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Message {
    performative: Performative,
    sender: Option<Aid>,
    receiver: Receiver,
    reply_to: Option<Aid>,
    content: Content,
    language: Option<Language>,
    // TODO: Implement these.
    // ontology: Option<Ontology>,
    // protocol: Option<Protocol>,
    // conversation_id: Option<String>,
    // reply_with: Option<String>,
    // in_reply_to: Option<String>,
    // reply_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Receiver {
    Single(Aid),
    Multiple(BTreeSet<Aid>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Performative {
    AcceptProposal,
    Agree,
    Cancel,
    Cfp,
    Confirm,
    Disconfirm,
    Failure,
    Inform,
    InformIf,
    InformRef,
    NotUnderstood,
    Propose,
    QueryIf,
    QueryRef,
    Refuse,
    RejectProposal,
    Request,
    RequestWhen,
    RequestWhenever,
    Subscribe,
    Proxy,
    Ropagate,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    Sl,
    Ccl,
    Kif,
    Rdf,
}
