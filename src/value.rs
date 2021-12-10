use std::{
    cmp::{Eq, Ordering},
    hash::{Hash, Hasher},
    iter::FromIterator,
    ops::{Index, IndexMut},
};

/// A `Value` to `Value` map.
///
/// This structure either uses a [BTreeMap](std::collections::BTreeMap) or the
/// [IndexMap](indexmap::IndexMap) internally.
/// The latter can be used by enabling the `indexmap` feature. This can be used
/// to preserve the order of the parsed map.
#[derive(Clone, Debug, Default)]
pub struct Map(pub MapInner);

impl Map {
    /// Creates a new, empty `Map`.
    pub fn new() -> Map {
        Default::default()
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if `self.len() == 0`, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    /// Inserts a new element, returning the previous element with this `key` if
    /// there was any.
    pub fn insert(&mut self, key: Value, value: Value) -> Option<Value> {
        self.0.insert(key, value)
    }

    /// Removes an element by its `key`.
    pub fn remove(&mut self, key: &Value) -> Option<Value> {
        self.0.remove(key)
    }

    /// Iterate all key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> + DoubleEndedIterator {
        self.0.iter()
    }

    /// Iterate all key-value pairs mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Value, &mut Value)> + DoubleEndedIterator {
        self.0.iter_mut()
    }

    /// Iterate all keys.
    pub fn keys(&self) -> impl Iterator<Item = &Value> + DoubleEndedIterator {
        self.0.keys()
    }

    /// Iterate all values.
    pub fn values(&self) -> impl Iterator<Item = &Value> + DoubleEndedIterator {
        self.0.values()
    }

    /// Iterate all values mutably.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Value> + DoubleEndedIterator {
        self.0.values_mut()
    }
}

impl FromIterator<(Value, Value)> for Map {
    fn from_iter<T: IntoIterator<Item = (Value, Value)>>(iter: T) -> Self {
        Map(MapInner::from_iter(iter))
    }
}

/// Note: equality is only given if both values and order of values match
impl Eq for Map {}

impl Hash for Map {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter().for_each(|x| x.hash(state));
    }
}

impl Index<&Value> for Map {
    type Output = Value;

    fn index(&self, index: &Value) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<&Value> for Map {
    fn index_mut(&mut self, index: &Value) -> &mut Self::Output {
        self.0.get_mut(index).expect("no entry found for key")
    }
}

impl Ord for Map {
    fn cmp(&self, other: &Map) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

/// Note: equality is only given if both values and order of values match
impl PartialEq for Map {
    fn eq(&self, other: &Map) -> bool {
        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl PartialOrd for Map {
    fn partial_cmp(&self, other: &Map) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

type MapInner = indexmap::IndexMap<Value, Value>;

/// A wrapper for a number, which can be either `f64` or `i64`.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
pub enum Number {
    Integer(i64),
    Float(Float),
}

/// A `Struct` to `Value` map.
#[derive(Clone, Debug, Default)]
pub struct Struct {
    pub name: Option<String>,
    pub prototype: Option<String>,
    pub fields: StructInner,
}

impl Struct {
    /// Creates a new, empty `Struct`.
    pub fn new(name: Option<String>, prototype: Option<String>) -> Struct {
        Struct {
            name,
            prototype,
            fields: Default::default(),
        }
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Returns `true` if `self.len() == 0`, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Inserts a new element, returning the previous element with this `key` if
    /// there was any.
    pub fn insert(&mut self, key: String, value: Value) -> Option<Value> {
        self.fields.insert(key, value)
    }

    /// Removes an element by its `key`.
    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.fields.remove(key)
    }

    /// Iterate all key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> + DoubleEndedIterator {
        self.fields.iter()
    }

    /// Iterate all key-value pairs mutably.
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&String, &mut Value)> + DoubleEndedIterator {
        self.fields.iter_mut()
    }

    /// Iterate all keys.
    pub fn keys(&self) -> impl Iterator<Item = &String> + DoubleEndedIterator {
        self.fields.keys()
    }

    /// Iterate all values.
    pub fn values(&self) -> impl Iterator<Item = &Value> + DoubleEndedIterator {
        self.fields.values()
    }

    /// Iterate all values mutably.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Value> + DoubleEndedIterator {
        self.fields.values_mut()
    }
}

impl FromIterator<(String, Value)> for Struct {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        Struct {
            name: None,
            fields: StructInner::from_iter(iter),
            prototype: None,
        }
    }
}

/// Note: equality is only given if both values and order of values match
impl Eq for Struct {}

impl Hash for Struct {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.iter().for_each(|x| x.hash(state));
    }
}

impl Index<&str> for Struct {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        &self.fields[index]
    }
}

impl IndexMut<&str> for Struct {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        self.fields.get_mut(index).expect("no entry found for key")
    }
}

impl Ord for Struct {
    fn cmp(&self, other: &Struct) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

/// Note: equality is only given if both values and order of values match
impl PartialEq for Struct {
    fn eq(&self, other: &Struct) -> bool {
        self.fields.len() == other.fields.len()
            && self.iter().zip(other.iter()).all(|(a, b)| a == b)
            && self.name == other.name
            && self.prototype == other.prototype
    }
}

impl PartialOrd for Struct {
    fn partial_cmp(&self, other: &Struct) -> Option<Ordering> {
        match self.name.cmp(&other.name) {
            Ordering::Equal => self.iter().partial_cmp(other.iter()),
            o => Some(o),
        }
    }
}

type StructInner = indexmap::IndexMap<String, Value>;

/// A wrapper for `f64`, which guarantees that the inner value
/// is finite and thus implements `Eq`, `Hash` and `Ord`.
#[derive(Copy, Clone, Debug)]
pub struct Float(f64);

impl Float {
    /// Construct a new `Float`.
    pub fn new(v: f64) -> Self {
        Float(v)
    }

    /// Returns the wrapped float.
    pub fn get(self) -> f64 {
        self.0
    }
}

impl Number {
    /// Construct a new number.
    pub fn new(v: impl Into<Number>) -> Self {
        v.into()
    }

    /// Returns the `f64` representation of the number regardless of whether the number is stored
    /// as a float or integer.
    ///
    /// # Example
    ///
    /// ```
    /// # use ron_parser::value::Number;
    /// let i = Number::new(5);
    /// let f = Number::new(2.0);
    /// assert_eq!(i.into_f64(), 5.0);
    /// assert_eq!(f.into_f64(), 2.0);
    /// ```
    pub fn into_f64(self) -> f64 {
        self.map_to(|i| i as f64, |f| f)
    }

    /// If the `Number` is a float, return it. Otherwise return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use ron_parser::value::Number;
    /// let i = Number::new(5);
    /// let f = Number::new(2.0);
    /// assert_eq!(i.as_f64(), None);
    /// assert_eq!(f.as_f64(), Some(2.0));
    /// ```
    pub fn as_f64(self) -> Option<f64> {
        self.map_to(|_| None, Some)
    }

    /// If the `Number` is an integer, return it. Otherwise return `None`.
    ///
    /// # Example
    ///
    /// ```
    /// # use ron_parser::value::Number;
    /// let i = Number::new(5);
    /// let f = Number::new(2.0);
    /// assert_eq!(i.as_i64(), Some(5));
    /// assert_eq!(f.as_i64(), None);
    /// ```
    pub fn as_i64(self) -> Option<i64> {
        self.map_to(Some, |_| None)
    }

    /// Map this number to a single type using the appropriate closure.
    ///
    /// # Example
    ///
    /// ```
    /// # use ron_parser::value::Number;
    /// let i = Number::new(5);
    /// let f = Number::new(2.0);
    /// assert!(i.map_to(|i| i > 3, |f| f > 3.0));
    /// assert!(!f.map_to(|i| i > 3, |f| f > 3.0));
    /// ```
    pub fn map_to<T>(
        self,
        integer_fn: impl FnOnce(i64) -> T,
        float_fn: impl FnOnce(f64) -> T,
    ) -> T {
        match self {
            Number::Integer(i) => integer_fn(i),
            Number::Float(Float(f)) => float_fn(f),
        }
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Number {
        Number::Float(Float(f))
    }
}

impl From<i64> for Number {
    fn from(i: i64) -> Number {
        Number::Integer(i)
    }
}

impl From<i32> for Number {
    fn from(i: i32) -> Number {
        Number::Integer(i64::from(i))
    }
}

// The following number conversion checks if the integer fits losslessly into an i64, before
// constructing a Number::Integer variant. If not, the conversion defaults to float.

impl From<u64> for Number {
    fn from(i: u64) -> Number {
        if i <= std::i64::MAX as u64 {
            Number::Integer(i as i64)
        } else {
            Number::new(i as f64)
        }
    }
}

/// Partial equality comparison
/// In order to be able to use `Number` as a mapping key, NaN floating values
/// wrapped in `Float` are equals to each other. It is not the case for
/// underlying `f64` values itself.
impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        self.0.is_nan() && other.0.is_nan() || self.0 == other.0
    }
}

/// Equality comparison
/// In order to be able to use `Float` as a mapping key, NaN floating values
/// wrapped in `Float` are equals to each other. It is not the case for
/// underlying `f64` values itself.
impl Eq for Float {}

impl Hash for Float {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0 as u64);
    }
}

/// Partial ordering comparison
/// In order to be able to use `Number` as a mapping key, NaN floating values
/// wrapped in `Number` are equals to each other and are less then any other
/// floating value. It is not the case for the underlying `f64` values themselves.
/// ```
/// use ron_parser::value::Number;
/// assert!(Number::new(std::f64::NAN) < Number::new(std::f64::NEG_INFINITY));
/// assert_eq!(Number::new(std::f64::NAN), Number::new(std::f64::NAN));
/// ```
impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => Some(Ordering::Equal),
            (true, false) => Some(Ordering::Less),
            (false, true) => Some(Ordering::Greater),
            _ => self.0.partial_cmp(&other.0),
        }
    }
}

/// Ordering comparison
/// In order to be able to use `Float` as a mapping key, NaN floating values
/// wrapped in `Float` are equals to each other and are less then any other
/// floating value. It is not the case for underlying `f64` values itself. See
/// the `PartialEq` implementation.
impl Ord for Float {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("Bug: Contract violation")
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Value {
    Bool(bool),
    Char(char),
    Map(Map),
    Struct(Struct),
    Number(Number),
    Option(Option<Box<Value>>),
    String(String),
    Seq(Vec<Value>),
    Tuple(Option<String>, Vec<Value>),
    Include(String),
    Unit,
}

impl Value {
    pub fn fmt_as_rust(&self) -> String {
        match self {
            Value::Bool(b) => format!("Value::Bool({})", b),
            Value::Char(c) => format!("Value::Char({})", c),
            Value::Map(m) => format!(
                "Value::Map(Map(indexmap!{{{}}}))",
                m.0.iter()
                    .map(|(k, v)| format!("{} => {}", k.fmt_as_rust(), v.fmt_as_rust()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Struct(s) => format!(
                "Value::Struct(Struct{{prototype:{}, name:{}, fields: indexmap!{{{}}} }})",
                match &s.prototype {
                    None => "None".to_string(),
                    Some(p) => format!("Some(\"{}\".to_string())", p),
                },
                match &s.name {
                    None => "None".to_string(),
                    Some(n) => format!("Some(\"{}\".to_string())", n),
                },
                s.fields
                    .iter()
                    .map(|(k, v)| format!("\"{}\".to_string() => {}", k, v.fmt_as_rust()))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            Value::Number(Number::Float(f)) => format!("Value::Number(Number::from({}))", f.0),
            Value::Number(Number::Integer(i)) => format!("Value::Number(Number::from({}))", i),
            Value::Option(None) => "Value::Option(None)".to_string(),
            Value::Option(Some(v)) => format!("Value::Option(Some({}))", v.fmt_as_rust()),
            Value::String(s) => format!("Value::String(\"{}\".to_string())", s),
            Value::Seq(s) => format!(
                "Value::Seq(vec![{}])",
                s.iter()
                    .map(|v| v.fmt_as_rust())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Tuple(name, t) => format!(
                "Value::Tuple({}, vec![{}])",
                match name {
                    None => "None".to_string(),
                    Some(p) => format!("Some(\"{}\".to_string())", p),
                },
                t.iter()
                    .map(|v| v.fmt_as_rust())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Include(s) => format!("Value::Include(\"{}\".to_string())", s),
            Value::Unit => "Value::Unit".to_string(),
        }
    }
}

impl From<Value> for ron::Value {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(b) => ron::Value::Bool(b),
            Value::Char(c) => ron::Value::Char(c),
            Value::Map(m) => ron::Value::Map(ron::value::Map(
                m.0.into_iter().map(|(k, v)| (k.into(), v.into())).collect(),
            )),
            Value::Struct(s) => ron::Value::Struct(ron::value::Struct {
                name: s.name,
                fields: s.fields.into_iter().map(|(k, v)| (k, v.into())).collect(),
            }),
            Value::Number(Number::Float(f)) => ron::Value::Number(ron::Number::from(f.0)),
            Value::Number(Number::Integer(i)) => ron::Value::Number(ron::Number::from(i)),
            Value::Option(None) => ron::Value::Option(None),
            Value::Option(Some(v)) => ron::Value::Option(Some(Box::new((*v).into()))),
            Value::String(s) => ron::Value::String(s),
            Value::Seq(s) => ron::Value::Seq(s.into_iter().map(ron::Value::from).collect()),
            Value::Tuple(_, t) => ron::Value::Tuple(t.into_iter().map(ron::Value::from).collect()),
            Value::Include(_) => ron::Value::Unit,
            Value::Unit => ron::Value::Unit,
        }
    }
}
