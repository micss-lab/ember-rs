use alloc::string::String;

use crate::acl::codec::{DecodeConcept, DecodeConstant, DecodeError};
use crate::acl::sl::{Concept, ConceptParameters};

struct ManagementOntology;

struct RegisterAction {
    agent: AmsAgentDescription,
}

struct AmsAgentDescription {
    name: Option<String>,
}

impl DecodeConcept for AmsAgentDescription {
    fn from_concept(concept: Concept) -> Result<Self, DecodeError> {
        use ConceptParameters::*;
        if concept.symbol != "ams-agent-description" {
            return Err(DecodeError::UnknownConcept(concept.symbol));
        }
        let name = match concept.parameters {
            Positional(items) => {
                if items.len() > 1 {
                    return Err(DecodeError::InvalidLength(items.len()));
                }
                items
                    .into_iter()
                    .next()
                    .map(DecodeConstant::from_term)
                    .transpose()?
            }
            ByName(params) => {
                let mut result = None;
                for (key, value) in params {
                    match key.as_slice() {
                        b"name" => result = Some(value),
                        _ => return Err(DecodeError::UnknownField(key)),
                    }
                }
                match result {
                    Some(name) => Some(DecodeConstant::from_term(name)?),
                    None => return Err(DecodeError::MissingField("name")),
                }
            }
        };
        Ok(AmsAgentDescription { name })
    }
}
