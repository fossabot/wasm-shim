use std::cell::OnceCell;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use cel_interpreter::objects::ValueType;
use cel_interpreter::{Context, Expression, Value};
use cel_parser::{Atom, RelationOp};
use serde::Deserialize;

use crate::attribute::Attribute;
use crate::policy::Policy;
use crate::policy_index::PolicyIndex;

#[derive(Deserialize, Debug, Clone)]
pub struct SelectorItem {
    // Selector of an attribute from the contextual properties provided by kuadrant
    // during request and connection processing
    pub selector: String,

    // If not set it defaults to `selector` field value as the descriptor key.
    #[serde(default)]
    pub key: Option<String>,

    // An optional value to use if the selector is not found in the context.
    // If not set and the selector is not found in the context, then no data is generated.
    #[serde(default)]
    pub default: Option<String>,

    #[serde(skip_deserializing)]
    path: OnceCell<Path>,
}

impl SelectorItem {
    pub fn compile(&self) -> Result<(), String> {
        self.path
            .set(self.selector.as_str().into())
            .map_err(|p| format!("Err on {p:?}"))
    }

    pub fn path(&self) -> &Path {
        self.path
            .get()
            .expect("SelectorItem wasn't previously compiled!")
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    tokens: Vec<String>,
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.tokens
                .iter()
                .map(|t| t.replace('.', "\\."))
                .collect::<Vec<String>>()
                .join(".")
        )
    }
}

impl From<&str> for Path {
    fn from(value: &str) -> Self {
        let mut token = String::new();
        let mut tokens: Vec<String> = Vec::new();
        let mut chars = value.chars();
        while let Some(ch) = chars.next() {
            match ch {
                '.' => {
                    tokens.push(token);
                    token = String::new();
                }
                '\\' => {
                    if let Some(next) = chars.next() {
                        token.push(next);
                    }
                }
                _ => token.push(ch),
            }
        }
        tokens.push(token);

        Self { tokens }
    }
}

impl Path {
    pub fn tokens(&self) -> Vec<&str> {
        self.tokens.iter().map(String::as_str).collect()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct StaticItem {
    pub value: String,
    pub key: String,
}

// Mutually exclusive struct fields
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Static(StaticItem),
    Selector(SelectorItem),
}

impl DataType {
    pub fn compile(&self) -> Result<(), String> {
        match self {
            DataType::Static(_) => Ok(()),
            DataType::Selector(selector) => selector.compile(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DataItem {
    #[serde(flatten)]
    pub item: DataType,
}

#[derive(Deserialize, PartialEq, Debug, Clone)]
pub enum WhenConditionOperator {
    #[serde(rename = "eq")]
    Equal,
    #[serde(rename = "neq")]
    NotEqual,
    #[serde(rename = "startswith")]
    StartsWith,
    #[serde(rename = "endswith")]
    EndsWith,
    #[serde(rename = "matches")]
    Matches,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PatternExpression {
    pub selector: String,
    pub operator: WhenConditionOperator,
    pub value: String,

    #[serde(skip_deserializing)]
    path: OnceCell<Path>,
    #[serde(skip_deserializing)]
    compiled: OnceCell<CelExpression>,
}

impl PatternExpression {
    pub fn compile(&self) -> Result<(), String> {
        self.path
            .set(self.selector.as_str().into())
            .map_err(|_| "Duh!")?;
        self.compiled
            .set(self.try_into()?)
            .map_err(|_| "Ooops".to_string())
    }
    pub fn path(&self) -> Vec<&str> {
        self.path
            .get()
            .expect("PatternExpression wasn't previously compiled!")
            .tokens()
    }

    pub fn eval(&self, raw_attribute: Vec<u8>) -> Result<bool, String> {
        let cel_type = &self.compiled.get().unwrap().cel_type;
        let value = match cel_type {
            ValueType::String => Value::String(Arc::new(Attribute::parse(raw_attribute)?)),
            ValueType::Int => Value::Int(Attribute::parse(raw_attribute)?),
            ValueType::UInt => Value::UInt(Attribute::parse(raw_attribute)?),
            ValueType::Float => Value::Float(Attribute::parse(raw_attribute)?),
            ValueType::Bytes => Value::Bytes(Arc::new(Attribute::parse(raw_attribute)?)),
            ValueType::Bool => Value::Bool(Attribute::parse(raw_attribute)?),
            ValueType::Timestamp => Value::Timestamp(Attribute::parse(raw_attribute)?),
            // todo: Impl support for parsing these two types… Tho List/Map of what?
            // ValueType::List => {}
            // ValueType::Map => {}
            _ => unimplemented!("Need support for {}", cel_type),
        };
        let mut ctx = Context::default();
        ctx.add_variable_from_value("attribute", value);
        Value::resolve(&self.compiled.get().unwrap().expression, &ctx)
            .map(|v| {
                if let Value::Bool(result) = v {
                    result
                } else {
                    false
                }
            })
            .map_err(|err| format!("Error evaluating {:?}: {}", self.compiled, err))
    }
}

struct CelExpression {
    expression: Expression,
    cel_type: ValueType,
}

impl Debug for CelExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CelExpression({}, {:?}", self.cel_type, self.expression)
    }
}

impl Clone for CelExpression {
    fn clone(&self) -> Self {
        Self {
            expression: self.expression.clone(),
            cel_type: match self.cel_type {
                ValueType::List => ValueType::List,
                ValueType::Map => ValueType::Map,
                ValueType::Function => ValueType::Function,
                ValueType::Int => ValueType::Int,
                ValueType::UInt => ValueType::UInt,
                ValueType::Float => ValueType::Float,
                ValueType::String => ValueType::String,
                ValueType::Bytes => ValueType::Bytes,
                ValueType::Bool => ValueType::Bool,
                ValueType::Duration => ValueType::Duration,
                ValueType::Timestamp => ValueType::Timestamp,
                ValueType::Null => ValueType::Null,
            },
        }
    }
}

impl TryFrom<&PatternExpression> for CelExpression {
    type Error = String;

    fn try_from(expression: &PatternExpression) -> Result<Self, Self::Error> {
        let cel_value = match cel_parser::parse(&expression.value) {
            Ok(exp) => match exp {
                Expression::Ident(ident) => Expression::Atom(Atom::String(ident)),
                Expression::Member(_, _) => {
                    Expression::Atom(Atom::String(expression.value.to_string().into()))
                }
                _ => exp,
            },
            Err(_) => Expression::Atom(Atom::String(expression.value.clone().into())),
        };
        let cel_type = type_of(&expression.selector).unwrap_or(match &cel_value {
            Expression::List(_) => Ok(ValueType::List),
            Expression::Map(_) => Ok(ValueType::Map),
            Expression::Atom(atom) => Ok(match atom {
                Atom::Int(_) => ValueType::Int,
                Atom::UInt(_) => ValueType::UInt,
                Atom::Float(_) => ValueType::Float,
                Atom::String(_) => ValueType::String,
                Atom::Bytes(_) => ValueType::Bytes,
                Atom::Bool(_) => ValueType::Bool,
                Atom::Null => ValueType::Null,
            }),
            _ => Err(format!("Unsupported CEL value: {cel_value:?}")),
        }?);

        let value = match cel_type {
            ValueType::Map => match expression.operator {
                WhenConditionOperator::Equal | WhenConditionOperator::NotEqual => {
                    if let Expression::Map(data) = cel_value {
                        Ok(Expression::Map(data))
                    } else {
                        Err(format!("Can't compare {cel_value:?} with a Map"))
                    }
                }
                _ => Err(format!(
                    "Unsupported operator {:?} on Map",
                    &expression.operator
                )),
            },
            ValueType::Int | ValueType::UInt | ValueType::Float => match expression.operator {
                WhenConditionOperator::Equal | WhenConditionOperator::NotEqual => {
                    if let Expression::Atom(atom) = &cel_value {
                        match atom {
                            Atom::Int(_) | Atom::UInt(_) | Atom::Float(_) => Ok(cel_value),
                            _ => Err(format!("Can't compare {cel_value:?} with a Number")),
                        }
                    } else {
                        Err(format!("Can't compare {cel_value:?} with a Number"))
                    }
                }
                _ => Err(format!(
                    "Unsupported operator {:?} on Number",
                    &expression.operator
                )),
            },
            ValueType::String => match &cel_value {
                Expression::Atom(Atom::String(_)) => Ok(cel_value),
                _ => Ok(Expression::Atom(Atom::String(Arc::new(
                    expression.value.clone(),
                )))),
            },
            ValueType::Bytes => match expression.operator {
                WhenConditionOperator::Equal | WhenConditionOperator::NotEqual => {
                    if let Expression::Atom(atom) = &cel_value {
                        match atom {
                            Atom::String(_str) => Ok(cel_value),
                            Atom::Bytes(_bytes) => Ok(cel_value),
                            _ => Err(format!("Can't compare {cel_value:?} with Bytes")),
                        }
                    } else {
                        Err(format!("Can't compare {cel_value:?} with Bytes"))
                    }
                }
                _ => Err(format!(
                    "Unsupported operator {:?} on Bytes",
                    &expression.operator
                )),
            },
            ValueType::Bool => match expression.operator {
                WhenConditionOperator::Equal | WhenConditionOperator::NotEqual => {
                    if let Expression::Atom(atom) = &cel_value {
                        match atom {
                            Atom::Bool(_) => Ok(cel_value),
                            _ => Err(format!("Can't compare {cel_value:?} with Bool")),
                        }
                    } else {
                        Err(format!("Can't compare {cel_value:?} with Bool"))
                    }
                }
                _ => Err(format!(
                    "Unsupported operator {:?} on Bool",
                    &expression.operator
                )),
            },
            ValueType::Timestamp => match expression.operator {
                WhenConditionOperator::Equal | WhenConditionOperator::NotEqual => {
                    if let Expression::Atom(atom) = &cel_value {
                        match atom {
                            Atom::String(_) => Ok(Expression::FunctionCall(
                                Expression::Ident("timestamp".to_string().into()).into(),
                                None,
                                [cel_value].to_vec(),
                            )),
                            _ => Err(format!("Can't compare {cel_value:?} with Timestamp")),
                        }
                    } else {
                        Err(format!("Can't compare {cel_value:?} with Bool"))
                    }
                }
                _ => Err(format!(
                    "Unsupported operator {:?} on Bytes",
                    &expression.operator
                )),
            },
            _ => Err(format!(
                "Still needs support for values of type `{cel_type}`"
            )),
        }?;

        let expression = match expression.operator {
            WhenConditionOperator::Equal => Expression::Relation(
                Expression::Ident(Arc::new("attribute".to_string())).into(),
                RelationOp::Equals,
                value.into(),
            ),
            WhenConditionOperator::NotEqual => Expression::Relation(
                Expression::Ident(Arc::new("attribute".to_string())).into(),
                RelationOp::NotEquals,
                value.into(),
            ),
            WhenConditionOperator::StartsWith => Expression::FunctionCall(
                Expression::Ident(Arc::new("startsWith".to_string())).into(),
                Some(Expression::Ident("attribute".to_string().into()).into()),
                [value].to_vec(),
            ),
            WhenConditionOperator::EndsWith => Expression::FunctionCall(
                Expression::Ident(Arc::new("endsWith".to_string())).into(),
                Some(Expression::Ident("attribute".to_string().into()).into()),
                [value].to_vec(),
            ),
            WhenConditionOperator::Matches => Expression::FunctionCall(
                Expression::Ident(Arc::new("matches".to_string())).into(),
                Some(Expression::Ident("attribute".to_string().into()).into()),
                [value].to_vec(),
            ),
        };

        Ok(Self {
            expression,
            cel_type,
        })
    }
}

pub fn type_of(path: &str) -> Option<ValueType> {
    match path {
        "request.time" => Some(ValueType::Timestamp),
        "request.id" => Some(ValueType::String),
        "request.protocol" => Some(ValueType::String),
        "request.scheme" => Some(ValueType::String),
        "request.host" => Some(ValueType::String),
        "request.method" => Some(ValueType::String),
        "request.path" => Some(ValueType::String),
        "request.url_path" => Some(ValueType::String),
        "request.query" => Some(ValueType::String),
        "request.referer" => Some(ValueType::String),
        "request.useragent" => Some(ValueType::String),
        "request.body" => Some(ValueType::String),
        "source.address" => Some(ValueType::String),
        "source.service" => Some(ValueType::String),
        "source.principal" => Some(ValueType::String),
        "source.certificate" => Some(ValueType::String),
        "destination.address" => Some(ValueType::String),
        "destination.service" => Some(ValueType::String),
        "destination.principal" => Some(ValueType::String),
        "destination.certificate" => Some(ValueType::String),
        "connection.requested_server_name" => Some(ValueType::String),
        "connection.tls_session.sni" => Some(ValueType::String),
        "connection.tls_version" => Some(ValueType::String),
        "connection.subject_local_certificate" => Some(ValueType::String),
        "connection.subject_peer_certificate" => Some(ValueType::String),
        "connection.dns_san_local_certificate" => Some(ValueType::String),
        "connection.dns_san_peer_certificate" => Some(ValueType::String),
        "connection.uri_san_local_certificate" => Some(ValueType::String),
        "connection.uri_san_peer_certificate" => Some(ValueType::String),
        "connection.sha256_peer_certificate_digest" => Some(ValueType::String),
        "ratelimit.domain" => Some(ValueType::String),
        "request.size" => Some(ValueType::Int),
        "source.port" => Some(ValueType::Int),
        "destination.port" => Some(ValueType::Int),
        "connection.id" => Some(ValueType::Int),
        "ratelimit.hits_addend" => Some(ValueType::Int),
        "request.headers" => Some(ValueType::Map),
        "request.context_extensions" => Some(ValueType::Map),
        "source.labels" => Some(ValueType::Map),
        "destination.labels" => Some(ValueType::Map),
        "filter_state" => Some(ValueType::Map),
        "connection.mtls" => Some(ValueType::Bool),
        "request.raw_body" => Some(ValueType::Bytes),
        "auth.identity" => Some(ValueType::Bytes),
        _ => None,
    }
}

pub struct FilterConfig {
    pub index: PolicyIndex,
    // Deny/Allow request when faced with an irrecoverable failure.
    pub failure_mode: FailureMode,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            index: PolicyIndex::new(),
            failure_mode: FailureMode::Deny,
        }
    }
}

impl TryFrom<PluginConfiguration> for FilterConfig {
    type Error = String;

    fn try_from(config: PluginConfiguration) -> Result<Self, Self::Error> {
        let mut index = PolicyIndex::new();

        for rlp in config.policies.iter() {
            for rule in &rlp.rules {
                for datum in &rule.data {
                    let result = datum.item.compile();
                    if result.is_err() {
                        return Err(result.err().unwrap());
                    }
                }
                for condition in &rule.conditions {
                    for pe in &condition.all_of {
                        let result = pe.compile();
                        if result.is_err() {
                            return Err(result.err().unwrap());
                        }
                    }
                }
            }
            for hostname in rlp.hostnames.iter() {
                index.insert(hostname, rlp.clone());
            }
        }

        Ok(Self {
            index,
            failure_mode: config.failure_mode,
        })
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FailureMode {
    Deny,
    Allow,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PluginConfiguration {
    #[serde(rename = "rateLimitPolicies")]
    pub policies: Vec<Policy>,
    // Deny/Allow request when faced with an irrecoverable failure.
    pub failure_mode: FailureMode,
}

#[cfg(test)]
mod test {
    use super::*;

    const CONFIG: &str = r#"{
        "failureMode": "deny",
        "rateLimitPolicies": [
        {
            "name": "rlp-ns-A/rlp-name-A",
            "domain": "rlp-ns-A/rlp-name-A",
            "service": "limitador-cluster",
            "hostnames": ["*.toystore.com", "example.com"],
            "rules": [
            {
                "conditions": [
                {
                    "allOf": [
                    {
                        "selector": "request.path",
                        "operator": "eq",
                        "value": "/admin/toy"
                    },
                    {
                        "selector": "request.method",
                        "operator": "eq",
                        "value": "POST"
                    },
                    {
                        "selector": "request.host",
                        "operator": "eq",
                        "value": "cars.toystore.com"
                    }]
                }],
                "data": [
                {
                    "static": {
                        "key": "rlp-ns-A/rlp-name-A",
                        "value": "1"
                    }
                },
                {
                    "selector": {
                        "selector": "auth.metadata.username"
                    }
                }]
            }]
        }]
    }"#;

    #[test]
    fn parse_config_happy_path() {
        let res = serde_json::from_str::<PluginConfiguration>(CONFIG);
        if let Err(ref e) = res {
            eprintln!("{e}");
        }
        assert!(res.is_ok());

        let filter_config = res.unwrap();
        assert_eq!(filter_config.policies.len(), 1);

        let rules = &filter_config.policies[0].rules;
        assert_eq!(rules.len(), 1);

        let conditions = &rules[0].conditions;
        assert_eq!(conditions.len(), 1);

        let all_of_conditions = &conditions[0].all_of;
        assert_eq!(all_of_conditions.len(), 3);

        let data_items = &rules[0].data;
        assert_eq!(data_items.len(), 2);

        // TODO(eastizle): DataItem does not implement PartialEq, add it only for testing?
        //assert_eq!(
        //    data_items[0],
        //    DataItem {
        //        item: DataType::Static(StaticItem {
        //            key: String::from("rlp-ns-A/rlp-name-A"),
        //            value: String::from("1")
        //        })
        //    }
        //);

        if let DataType::Static(static_item) = &data_items[0].item {
            assert_eq!(static_item.key, "rlp-ns-A/rlp-name-A");
            assert_eq!(static_item.value, "1");
        } else {
            panic!();
        }

        if let DataType::Selector(selector_item) = &data_items[1].item {
            assert_eq!(selector_item.selector, "auth.metadata.username");
            assert!(selector_item.key.is_none());
            assert!(selector_item.default.is_none());
        } else {
            panic!();
        }
    }

    #[test]
    fn parse_config_min() {
        let config = r#"{
            "failureMode": "deny",
            "rateLimitPolicies": []
        }"#;
        let res = serde_json::from_str::<PluginConfiguration>(config);
        if let Err(ref e) = res {
            eprintln!("{e}");
        }
        assert!(res.is_ok());

        let filter_config = res.unwrap();
        assert_eq!(filter_config.policies.len(), 0);
    }

    #[test]
    fn parse_config_data_selector() {
        let config = r#"{
            "failureMode": "deny",
            "rateLimitPolicies": [
            {
                "name": "rlp-ns-A/rlp-name-A",
                "domain": "rlp-ns-A/rlp-name-A",
                "service": "limitador-cluster",
                "hostnames": ["*.toystore.com", "example.com"],
                "rules": [
                {
                    "data": [
                    {
                        "selector": {
                            "selector": "my.selector.path",
                            "key": "mykey",
                            "default": "my_selector_default_value"
                        }
                    }]
                }]
            }]
        }"#;
        let res = serde_json::from_str::<PluginConfiguration>(config);
        if let Err(ref e) = res {
            eprintln!("{e}");
        }
        assert!(res.is_ok());

        let filter_config = res.unwrap();
        assert_eq!(filter_config.policies.len(), 1);

        let rules = &filter_config.policies[0].rules;
        assert_eq!(rules.len(), 1);

        let data_items = &rules[0].data;
        assert_eq!(data_items.len(), 1);

        if let DataType::Selector(selector_item) = &data_items[0].item {
            assert_eq!(selector_item.selector, "my.selector.path");
            assert_eq!(selector_item.key.as_ref().unwrap(), "mykey");
            assert_eq!(
                selector_item.default.as_ref().unwrap(),
                "my_selector_default_value"
            );
        } else {
            panic!();
        }
    }

    #[test]
    fn parse_config_condition_selector_operators() {
        let config = r#"{
            "failureMode": "deny",
            "rateLimitPolicies": [
            {
                "name": "rlp-ns-A/rlp-name-A",
                "domain": "rlp-ns-A/rlp-name-A",
                "service": "limitador-cluster",
                "hostnames": ["*.toystore.com", "example.com"],
                "rules": [
                {
                    "conditions": [
                    {
                        "allOf": [
                        {
                            "selector": "request.path",
                            "operator": "eq",
                            "value": "/admin/toy"
                        },
                        {
                            "selector": "request.method",
                            "operator": "neq",
                            "value": "POST"
                        },
                        {
                            "selector": "request.host",
                            "operator": "startswith",
                            "value": "cars."
                        },
                        {
                            "selector": "request.host",
                            "operator": "endswith",
                            "value": ".com"
                        },
                        {
                            "selector": "request.host",
                            "operator": "matches",
                            "value": "*.com"
                        }]
                    }],
                    "data": [ { "selector": { "selector": "my.selector.path" } }]
                }]
            }]
        }"#;
        let res = serde_json::from_str::<PluginConfiguration>(config);
        if let Err(ref e) = res {
            eprintln!("{e}");
        }
        assert!(res.is_ok());

        let filter_config = res.unwrap();
        assert_eq!(filter_config.policies.len(), 1);

        let rules = &filter_config.policies[0].rules;
        assert_eq!(rules.len(), 1);

        let conditions = &rules[0].conditions;
        assert_eq!(conditions.len(), 1);

        let all_of_conditions = &conditions[0].all_of;
        assert_eq!(all_of_conditions.len(), 5);

        let expected_conditions = [
            // selector, value, operator
            ("request.path", "/admin/toy", WhenConditionOperator::Equal),
            ("request.method", "POST", WhenConditionOperator::NotEqual),
            ("request.host", "cars.", WhenConditionOperator::StartsWith),
            ("request.host", ".com", WhenConditionOperator::EndsWith),
            ("request.host", "*.com", WhenConditionOperator::Matches),
        ];

        for i in 0..expected_conditions.len() {
            assert_eq!(all_of_conditions[i].selector, expected_conditions[i].0);
            assert_eq!(all_of_conditions[i].value, expected_conditions[i].1);
            assert_eq!(all_of_conditions[i].operator, expected_conditions[i].2);
        }
    }

    #[test]
    fn parse_config_conditions_optional() {
        let config = r#"{
            "failureMode": "deny",
            "rateLimitPolicies": [
            {
                "name": "rlp-ns-A/rlp-name-A",
                "domain": "rlp-ns-A/rlp-name-A",
                "service": "limitador-cluster",
                "hostnames": ["*.toystore.com", "example.com"],
                "rules": [
                {
                    "data": [
                    {
                        "static": {
                            "key": "rlp-ns-A/rlp-name-A",
                            "value": "1"
                        }
                    },
                    {
                        "selector": {
                            "selector": "auth.metadata.username"
                        }
                    }]
                }]
            }]
        }"#;
        let res = serde_json::from_str::<PluginConfiguration>(config);
        if let Err(ref e) = res {
            eprintln!("{e}");
        }
        assert!(res.is_ok());

        let filter_config = res.unwrap();
        assert_eq!(filter_config.policies.len(), 1);

        let rules = &filter_config.policies[0].rules;
        assert_eq!(rules.len(), 1);

        let conditions = &rules[0].conditions;
        assert_eq!(conditions.len(), 0);
    }

    #[test]
    fn parse_config_invalid_data() {
        // data item fields are mutually exclusive
        let bad_config = r#"{
        "failureMode": "deny",
        "rateLimitPolicies": [
        {
            "name": "rlp-ns-A/rlp-name-A",
            "domain": "rlp-ns-A/rlp-name-A",
            "service": "limitador-cluster",
            "hostnames": ["*.toystore.com", "example.com"],
            "rules": [
            {
                "data": [
                {
                    "static": {
                        "key": "rlp-ns-A/rlp-name-A",
                        "value": "1"
                    },
                    "selector": {
                        "selector": "auth.metadata.username"
                    }
                }]
            }]
        }]
        }"#;
        let res = serde_json::from_str::<PluginConfiguration>(bad_config);
        assert!(res.is_err());

        // data item unknown fields are forbidden
        let bad_config = r#"{
        "failureMode": "deny",
        "rateLimitPolicies": [
        {
            "name": "rlp-ns-A/rlp-name-A",
            "domain": "rlp-ns-A/rlp-name-A",
            "service": "limitador-cluster",
            "hostnames": ["*.toystore.com", "example.com"],
            "rules": [
            {
                "data": [
                {
                    "unknown": {
                        "key": "rlp-ns-A/rlp-name-A",
                        "value": "1"
                    }
                }]
            }]
        }]
        }"#;
        let res = serde_json::from_str::<PluginConfiguration>(bad_config);
        assert!(res.is_err());

        // condition selector operator unknown
        let bad_config = r#"{
            "failureMode": "deny",
            "rateLimitPolicies": [
            {
                "name": "rlp-ns-A/rlp-name-A",
                "domain": "rlp-ns-A/rlp-name-A",
                "service": "limitador-cluster",
                "hostnames": ["*.toystore.com", "example.com"],
                "rules": [
                {
                    "conditions": [
                    {
                        "allOf": [
                        {
                            "selector": "request.path",
                            "operator": "unknown",
                            "value": "/admin/toy"
                        }]
                    }],
                    "data": [ { "selector": { "selector": "my.selector.path" } }]
                }]
            }]
        }"#;
        let res = serde_json::from_str::<PluginConfiguration>(bad_config);
        assert!(res.is_err());
    }

    #[test]
    fn filter_config_from_configuration() {
        let res = serde_json::from_str::<PluginConfiguration>(CONFIG);
        if let Err(ref e) = res {
            eprintln!("{e}");
        }
        assert!(res.is_ok());

        let result = FilterConfig::try_from(res.unwrap());
        let filter_config = result.expect("That didn't work");
        let rlp_option = filter_config.index.get_longest_match_policy("example.com");
        assert!(rlp_option.is_some());

        let rlp_option = filter_config
            .index
            .get_longest_match_policy("test.toystore.com");
        assert!(rlp_option.is_some());

        let rlp_option = filter_config.index.get_longest_match_policy("unknown");
        assert!(rlp_option.is_none());
    }

    #[test]
    fn path_tokenizes_with_escaping_basic() {
        let path: Path = r"one\.two..three\\\\.four\\\.\five.".into();
        assert_eq!(
            path.tokens(),
            vec!["one.two", "", r"three\\", r"four\.five", ""]
        );
    }

    #[test]
    fn path_tokenizes_with_escaping_ends_with_separator() {
        let path: Path = r"one.".into();
        assert_eq!(path.tokens(), vec!["one", ""]);
    }

    #[test]
    fn path_tokenizes_with_escaping_ends_with_escape() {
        let path: Path = r"one\".into();
        assert_eq!(path.tokens(), vec!["one"]);
    }

    #[test]
    fn path_tokenizes_with_escaping_starts_with_separator() {
        let path: Path = r".one".into();
        assert_eq!(path.tokens(), vec!["", "one"]);
    }

    #[test]
    fn path_tokenizes_with_escaping_starts_with_escape() {
        let path: Path = r"\one".into();
        assert_eq!(path.tokens(), vec!["one"]);
    }

    mod pattern_expressions {
        use crate::configuration::{PatternExpression, WhenConditionOperator};

        #[test]
        fn test_legacy_string() {
            let p = PatternExpression {
                selector: "request.id".to_string(),
                operator: WhenConditionOperator::Equal,
                value: "request_id".to_string(),
                path: Default::default(),
                compiled: Default::default(),
            };
            p.compile().expect("Should compile fine!");
            assert_eq!(
                p.eval("request_id".as_bytes().to_vec()),
                Ok(true),
                "Expression: {:?}",
                p.compiled.get()
            )
        }

        #[test]
        fn test_proper_string() {
            let p = PatternExpression {
                selector: "request.id".to_string(),
                operator: WhenConditionOperator::Equal,
                value: "\"request_id\"".to_string(),
                path: Default::default(),
                compiled: Default::default(),
            };
            p.compile().expect("Should compile fine!");
            assert_eq!(
                p.eval("request_id".as_bytes().to_vec()),
                Ok(true),
                "Expression: {:?}",
                p.compiled.get()
            )
        }

        #[test]
        fn test_proper_int_as_string() {
            let p = PatternExpression {
                selector: "request.id".to_string(),
                operator: WhenConditionOperator::Equal,
                value: "123".to_string(),
                path: Default::default(),
                compiled: Default::default(),
            };
            p.compile().expect("Should compile fine!");
            assert_eq!(
                p.eval("123".as_bytes().to_vec()),
                Ok(true),
                "Expression: {:?}",
                p.compiled.get()
            )
        }

        #[test]
        fn test_proper_string_inferred() {
            let p = PatternExpression {
                selector: "foobar".to_string(),
                operator: WhenConditionOperator::Equal,
                value: "\"123\"".to_string(),
                path: Default::default(),
                compiled: Default::default(),
            };
            p.compile().expect("Should compile fine!");
            assert_eq!(
                p.eval("123".as_bytes().to_vec()),
                Ok(true),
                "Expression: {:?}",
                p.compiled.get()
            )
        }

        #[test]
        fn test_int() {
            let p = PatternExpression {
                selector: "destination.port".to_string(),
                operator: WhenConditionOperator::Equal,
                value: "8080".to_string(),
                path: Default::default(),
                compiled: Default::default(),
            };
            p.compile().expect("Should compile fine!");
            assert_eq!(
                p.eval(8080_i64.to_le_bytes().to_vec()),
                Ok(true),
                "Expression: {:?}",
                p.compiled.get()
            )
        }

        #[test]
        fn test_int_inferred() {
            let p = PatternExpression {
                selector: "foobar".to_string(),
                operator: WhenConditionOperator::Equal,
                value: "8080".to_string(),
                path: Default::default(),
                compiled: Default::default(),
            };
            p.compile().expect("Should compile fine!");
            assert_eq!(
                p.eval(8080_i64.to_le_bytes().to_vec()),
                Ok(true),
                "Expression: {:?}",
                p.compiled.get()
            )
        }

        #[test]
        fn test_float_inferred() {
            let p = PatternExpression {
                selector: "foobar".to_string(),
                operator: WhenConditionOperator::Equal,
                value: "1.0".to_string(),
                path: Default::default(),
                compiled: Default::default(),
            };
            p.compile().expect("Should compile fine!");
            assert_eq!(
                p.eval(1_f64.to_le_bytes().to_vec()),
                Ok(true),
                "Expression: {:?}",
                p.compiled.get()
            )
        }

        #[test]
        fn test_bool() {
            let p = PatternExpression {
                selector: "connection.mtls".to_string(),
                operator: WhenConditionOperator::Equal,
                value: "true".to_string(),
                path: Default::default(),
                compiled: Default::default(),
            };
            p.compile().expect("Should compile fine!");
            assert_eq!(
                p.eval((true as u8).to_le_bytes().to_vec()),
                Ok(true),
                "Expression: {:?}",
                p.compiled.get()
            )
        }

        #[test]
        fn test_timestamp() {
            let p = PatternExpression {
                selector: "request.time".to_string(),
                operator: WhenConditionOperator::Equal,
                value: "2023-05-28T00:00:00+00:00".to_string(),
                path: Default::default(),
                compiled: Default::default(),
            };
            p.compile().expect("Should compile fine!");
            assert_eq!(
                p.eval(1685232000000000000_i64.to_le_bytes().to_vec()),
                Ok(true),
                "Expression: {:?}",
                p.compiled.get()
            )
        }
    }
}
