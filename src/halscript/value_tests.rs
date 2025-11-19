#[cfg(test)]
mod tests {
    use super::super::value::Value;
    use super::*;

    #[test]
    fn test_value_add() {
        let a = Value::Number(10);
        let b = Value::Number(20);
        assert_eq!(a.add(&b).unwrap(), Value::Number(30));

        let s1 = Value::String("hello".into());
        let s2 = Value::String(" world".into());
        assert_eq!(s1.add(&s2).unwrap(), Value::String("hello world".into()));
    }

    #[test]
    fn test_value_arithmetic() {
        let a = Value::Number(10);
        let b = Value::Number(3);

        assert_eq!(a.sub(&b).unwrap(), Value::Number(7));
        assert_eq!(a.mul(&b).unwrap(), Value::Number(30));
        assert_eq!(a.div(&b).unwrap(), Value::Number(3));
    }

    #[test]
    fn test_value_division_by_zero() {
        let a = Value::Number(10);
        let b = Value::Number(0);

        assert!(a.div(&b).is_err());
    }

    #[test]
    fn test_value_truthy() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
        assert!(!Value::Null.is_truthy());
        assert!(Value::Number(1).is_truthy());
        assert!(!Value::Number(0).is_truthy());
        assert!(Value::String("hello".into()).is_truthy());
        assert!(!Value::String("".into()).is_truthy());
    }

    #[test]
    fn test_value_compare() {
        use core::cmp::Ordering;

        let a = Value::Number(10);
        let b = Value::Number(20);

        assert_eq!(a.compare(&b), Some(Ordering::Less));
        assert_eq!(b.compare(&a), Some(Ordering::Greater));

        let c = Value::Number(10);
        assert_eq!(a.compare(&c), Some(Ordering::Equal));
    }

    #[test]
    fn test_value_to_string() {
        assert_eq!(Value::Number(42).to_string(), "42");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Null.to_string(), "null");
        assert_eq!(Value::String("test".into()).to_string(), "test");

        let arr = Value::Array(vec![Value::Number(1), Value::Number(2)]);
        assert_eq!(arr.to_string(), "[1, 2]");
    }
}
