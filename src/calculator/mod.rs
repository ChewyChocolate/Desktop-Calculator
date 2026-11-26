//! UI-independent expression engine. Decimal arithmetic stays decimal; scientific
//! operations explicitly cross into IEEE-754 double precision.
use rust_decimal::{Decimal, prelude::ToPrimitive};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    Decimal(Decimal),
    Scientific(f64),
}
impl Value {
    pub fn as_f64(self) -> f64 {
        match self {
            Self::Decimal(d) => d.to_f64().unwrap_or(0.0),
            Self::Scientific(f) => f,
        }
    }
    fn scientific(f: f64) -> Result<Self, CalcError> {
        if f.is_finite() {
            Ok(Self::Scientific(if f == 0.0 { 0.0 } else { f }))
        } else {
            Err(CalcError("Result is outside the real, finite range"))
        }
    }
    pub fn text(self) -> String {
        match self {
            Self::Decimal(d) => d.normalize().to_string(),
            Self::Scientific(f) => {
                if f == 0.0 {
                    return "0".into();
                }
                let exponent = f.abs().log10().floor() as i32;
                if !(-6..15).contains(&exponent) {
                    format!("{f:.14e}")
                        .split_once('e')
                        .map(|(a, b)| {
                            format!(
                                "{}e{}",
                                a.trim_end_matches('0').trim_end_matches('.'),
                                b.parse::<i32>().unwrap_or(0)
                            )
                        })
                        .unwrap()
                } else {
                    let text = format!("{:.*}", (14 - exponent).max(0) as usize, f);
                    if text.contains('.') {
                        text.trim_end_matches('0').trim_end_matches('.').to_owned()
                    } else {
                        text
                    }
                }
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct CalcError(pub &'static str);
impl CalcError {
    pub fn is_incomplete(&self) -> bool {
        matches!(
            self.0,
            "Enter a number or complete the expression"
                | "Missing closing parenthesis"
                | "A function needs parentheses"
                | "Incomplete scientific notation"
        )
    }
}
impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}
impl std::error::Error for CalcError {}

#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Angle {
    #[default]
    Degrees,
    Radians,
}
#[derive(Clone, Debug)]
pub struct Evaluation {
    pub value: Value,
    pub repeat: Option<(char, Value)>,
}
#[derive(Debug)]
enum Node {
    Number(Value),
    Unary(char, Box<Node>),
    Binary(char, Box<Node>, Box<Node>),
    Function(String, Box<Node>),
}

pub fn evaluate(expression: &str, angle: Angle) -> Result<Evaluation, CalcError> {
    if expression.len() > 2048 {
        return Err(CalcError("Expression is too long (limit: 2,048 bytes)"));
    }
    if expression
        .chars()
        .filter(|c| "+-*/^%!()".contains(*c))
        .count()
        > 256
    {
        return Err(CalcError("Too many operations (limit: 256)"));
    }
    let normalized = expression
        .replace('×', "*")
        .replace('÷', "/")
        .replace('−', "-")
        .replace('π', "pi");
    let mut parser = Parser {
        s: normalized.as_bytes(),
        pos: 0,
        depth: 0,
    };
    let root = parser.expr(0)?;
    parser.space();
    if parser.pos != parser.s.len() {
        return Err(CalcError(
            "Unexpected input; use an operator between values",
        ));
    }
    let value = root.eval(angle)?;
    let repeat = if let Node::Binary(op, _, rhs) = &root {
        Some((*op, rhs.eval(angle)?))
    } else {
        None
    };
    Ok(Evaluation { value, repeat })
}
pub fn binary(op: char, a: Value, b: Value) -> Result<Value, CalcError> {
    if op == '/' && b.as_f64() == 0.0 {
        return Err(CalcError("Cannot divide by zero"));
    }
    if let (Value::Decimal(a), Value::Decimal(b)) = (a, b) {
        let result = match op {
            '+' => a.checked_add(b),
            '-' => a.checked_sub(b),
            '*' => a.checked_mul(b),
            '/' => a.checked_div(b),
            '^' => {
                let mut ok = true;
                if b.fract().is_zero() && b.abs() <= Decimal::from(1000) {
                    let n = b.to_i32().unwrap();
                    let mut result = Decimal::ONE;
                    let mut base = a;
                    let mut power = n.unsigned_abs();
                    while power > 0 {
                        if power & 1 == 1 {
                            match result.checked_mul(base) {
                                Some(v) => result = v,
                                None => {
                                    ok = false;
                                    break;
                                }
                            }
                        }
                        power >>= 1;
                        if power > 0 {
                            match base.checked_mul(base) {
                                Some(v) => base = v,
                                None => {
                                    ok = false;
                                    break;
                                }
                            }
                        }
                    }
                    if !ok {
                        None
                    } else if n < 0 {
                        Decimal::ONE.checked_div(result)
                    } else {
                        Some(result)
                    }
                } else {
                    None
                }
            }
            _ => None,
        };
        let underflow = result.is_some_and(|r| r.is_zero())
            && !a.is_zero()
            && !b.is_zero()
            && matches!(op, '*' | '/' | '^');
        if let Some(d) = result
            && !underflow
        {
            return Ok(Value::Decimal(d));
        }
        // Promote decimal overflow/underflow into IEEE-754 scientific notation.
    }
    let (a, b) = (a.as_f64(), b.as_f64());
    let f = match op {
        '+' => a + b,
        '-' => a - b,
        '*' => a * b,
        '/' => a / b,
        '^' => a.powf(b),
        _ => return Err(CalcError("Unknown operator")),
    };
    if f == 0.0 && a != 0.0 && b != 0.0 && matches!(op, '*' | '/' | '^') {
        return Err(CalcError("Number is below scientific precision"));
    }
    Value::scientific(f)
}
impl Node {
    fn eval(&self, angle: Angle) -> Result<Value, CalcError> {
        match self {
            Self::Number(n) => Ok(*n),
            Self::Binary(op, a, b) => binary(*op, a.eval(angle)?, b.eval(angle)?),
            Self::Unary(op, n) => {
                let v = n.eval(angle)?;
                match op {
                    '-' => binary('-', Value::Decimal(Decimal::ZERO), v),
                    '%' => binary('/', v, Value::Decimal(Decimal::from(100))),
                    '!' => {
                        if let Value::Decimal(d) = v
                            && (d < Decimal::ZERO || d > Decimal::from(170) || !d.fract().is_zero())
                        {
                            return Err(CalcError("Factorial needs a whole number from 0 to 170"));
                        }
                        let f = v.as_f64();
                        if !(0.0..=170.0).contains(&f) || f.fract() != 0.0 {
                            return Err(CalcError("Factorial needs a whole number from 0 to 170"));
                        }
                        if f <= 27.0 {
                            let d = (1..=f as u32).fold(Decimal::ONE, |a, b| a * Decimal::from(b));
                            Ok(Value::Decimal(d))
                        } else {
                            Value::scientific((1..=f as u32).fold(1.0, |a, b| a * f64::from(b)))
                        }
                    }
                    _ => Ok(v),
                }
            }
            Self::Function(name, n) => {
                let v = n.eval(angle)?.as_f64();
                let a = if angle == Angle::Degrees {
                    v.to_radians()
                } else {
                    v
                };
                let inverse = |v: f64| {
                    if angle == Angle::Degrees {
                        v.to_degrees()
                    } else {
                        v
                    }
                };
                let r = match name.as_str() {
                    "sqrt" if v >= 0.0 => v.sqrt(),
                    "cbrt" => v.cbrt(),
                    "sin" => a.sin(),
                    "cos" => a.cos(),
                    "tan" => {
                        if a.cos().abs() < 1e-14 {
                            return Err(CalcError("Tangent is undefined at this angle"));
                        }
                        a.tan()
                    }
                    "asin" => inverse(v.asin()),
                    "acos" => inverse(v.acos()),
                    "atan" => inverse(v.atan()),
                    "ln" => v.ln(),
                    "log" => v.log10(),
                    "exp" => v.exp(),
                    "abs" => v.abs(),
                    "sqrt" => return Err(CalcError("Square root needs a non-negative value")),
                    _ => return Err(CalcError("Unknown function")),
                };
                Value::scientific(r)
            }
        }
    }
}
struct Parser<'a> {
    s: &'a [u8],
    pos: usize,
    depth: usize,
}
impl Parser<'_> {
    fn space(&mut self) {
        while self.s.get(self.pos).is_some_and(u8::is_ascii_whitespace) {
            self.pos += 1;
        }
    }
    fn peek(&mut self) -> Option<u8> {
        self.space();
        self.s.get(self.pos).copied()
    }
    fn expr(&mut self, min: u8) -> Result<Node, CalcError> {
        self.depth += 1;
        if self.depth > 64 {
            return Err(CalcError("Expression is nested too deeply"));
        }
        let mut lhs = match self.peek() {
            Some(b'+' | b'-') => {
                let op = self.s[self.pos] as char;
                self.pos += 1;
                Node::Unary(op, Box::new(self.expr(5)?))
            }
            Some(b'(') => {
                self.pos += 1;
                let node = self.expr(0)?;
                self.close()?;
                node
            }
            Some(c) if c.is_ascii_digit() || c == b'.' => self.number()?,
            Some(c) if c.is_ascii_alphabetic() => {
                let start = self.pos;
                while self.s.get(self.pos).is_some_and(u8::is_ascii_alphabetic) {
                    self.pos += 1;
                }
                let name = std::str::from_utf8(&self.s[start..self.pos]).unwrap();
                match name {
                    "pi" => Node::Number(Value::Scientific(std::f64::consts::PI)),
                    "e" => Node::Number(Value::Scientific(std::f64::consts::E)),
                    _ => {
                        if self.peek() != Some(b'(') {
                            return Err(CalcError("A function needs parentheses"));
                        }
                        self.pos += 1;
                        let arg = self.expr(0)?;
                        self.close()?;
                        Node::Function(name.into(), Box::new(arg))
                    }
                }
            }
            None => return Err(CalcError("Enter a number or complete the expression")),
            _ => return Err(CalcError("Expected a number or function")),
        };
        while let Some(op) = self.peek() {
            if op == b'%' || op == b'!' {
                if 9 < min {
                    break;
                }
                self.pos += 1;
                lhs = Node::Unary(op as char, Box::new(lhs));
                continue;
            }
            let (l, r) = match op {
                b'+' | b'-' => (1, 2),
                b'*' | b'/' => (3, 4),
                b'^' => (7, 6),
                _ => break,
            };
            if l < min {
                break;
            }
            self.pos += 1;
            lhs = Node::Binary(op as char, Box::new(lhs), Box::new(self.expr(r)?));
        }
        self.depth -= 1;
        Ok(lhs)
    }
    fn close(&mut self) -> Result<(), CalcError> {
        if self.peek() == Some(b')') {
            self.pos += 1;
            Ok(())
        } else if self.peek().is_none() {
            Err(CalcError("Missing closing parenthesis"))
        } else {
            Err(CalcError("Expected a closing parenthesis"))
        }
    }
    fn number(&mut self) -> Result<Node, CalcError> {
        let start = self.pos;
        while self
            .s
            .get(self.pos)
            .is_some_and(|c| c.is_ascii_digit() || *c == b'.')
        {
            self.pos += 1;
        }
        if self
            .s
            .get(self.pos)
            .is_some_and(|c| *c == b'e' || *c == b'E')
            && !self
                .s
                .get(self.pos + 1)
                .is_some_and(u8::is_ascii_alphabetic)
        {
            self.pos += 1;
            if self
                .s
                .get(self.pos)
                .is_some_and(|c| *c == b'+' || *c == b'-')
            {
                self.pos += 1;
            }
            let exp_start = self.pos;
            while self.s.get(self.pos).is_some_and(u8::is_ascii_digit) {
                self.pos += 1;
            }
            if self.pos == exp_start {
                return Err(CalcError("Incomplete scientific notation"));
            }
        }
        let token = std::str::from_utf8(&self.s[start..self.pos]).unwrap();
        let d = if token.contains(['e', 'E']) {
            Decimal::from_scientific(token)
        } else {
            Decimal::from_str_exact(token)
        };
        // Promote out-of-range literals and explicit scientific notation to f64.
        let has_nonzero_mantissa = token
            .split(['e', 'E'])
            .next()
            .is_some_and(|m| m.chars().any(|c| matches!(c, '1'..='9')));
        if d.is_err() || (d.as_ref().is_ok_and(|v| v.is_zero()) && has_nonzero_mantissa) {
            let f = token
                .parse::<f64>()
                .map_err(|_| CalcError("Invalid number"))?;
            if f == 0.0 && has_nonzero_mantissa {
                return Err(CalcError("Number is below scientific precision"));
            }
            return Value::scientific(f).map(Node::Number);
        }
        d.map(|d| Node::Number(Value::Decimal(d)))
            .map_err(|_| CalcError("Invalid number or decimal range exceeded"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn calc(s: &str) -> String {
        evaluate(s, Angle::Degrees).unwrap().value.text()
    }
    #[test]
    fn scientific_format_and_range() {
        assert_eq!(Value::Scientific(1e14).text(), "100000000000000");
        assert_eq!(calc("1e100*2"), "2e100");
        assert_eq!(calc("1e-100*2"), "2e-100");
        assert_eq!(calc("10^30"), "1e30");
        assert_eq!(calc("1e-28/10"), "1e-29");
        assert_eq!(calc("0.00000000000000000000000000001"), "1e-29");
        assert!(evaluate("1e-999", Angle::Degrees).is_err());
        assert!(evaluate("1e", Angle::Degrees).unwrap_err().is_incomplete());
        assert!(evaluate("1e-", Angle::Degrees).unwrap_err().is_incomplete());
        assert!(evaluate("1e+", Angle::Degrees).unwrap_err().is_incomplete());
    }
    #[test]
    fn decimal_accuracy() {
        assert_eq!(calc("0.1+0.2"), "0.3");
        assert_eq!(calc("1.25*8"), "10");
        assert_eq!(calc("1/8"), "0.125");
    }
    #[test]
    fn precedence() {
        assert_eq!(calc("2+3*4"), "14");
        assert_eq!(calc("(2+3)*4"), "20");
        assert_eq!(calc("2^3^2"), "512");
        assert_eq!(calc("-2^2"), "-4");
        assert_eq!(calc("2^-3"), "0.125");
    }
    #[test]
    fn percentage_is_postfix_division_in_every_context() {
        for (expression, expected) in [
            ("200*10%", "20"),
            ("200+10%", "200.1"),
            ("200-10%", "199.9"),
            ("50%", "0.5"),
            ("0%", "0"),
        ] {
            assert_eq!(calc(expression), expected, "{expression}");
        }
    }
    #[test]
    fn unary_operations() {
        assert_eq!(calc("200*10%"), "20");
        assert_eq!(calc("5!"), "120");
        assert_eq!(calc("0!"), "1");
        assert_eq!(calc("sqrt(81)"), "9");
        assert_eq!(calc("cbrt(-8)"), "-2");
    }
    #[test]
    fn scientific() {
        assert!((evaluate("sin(30)", Angle::Degrees).unwrap().value.as_f64() - 0.5).abs() < 1e-14);
        assert_eq!(calc("log(1000)"), "3");
        assert_eq!(calc("asin(1)"), "90");
        assert_eq!(calc("1e-3+2e-3"), "0.003");
        assert_eq!(calc("27!"), "10888869450418352160768000000");
    }
    #[test]
    fn errors() {
        for s in [
            "1/0", "sqrt(-1)", "ln(0)", "acos(2)", "tan(90)", "(-1)!", "1.5!", "171!", "1..2",
            "2(3)", "(1+2", "foo(2)", "1e999", "1+",
        ] {
            assert!(evaluate(s, Angle::Degrees).is_err(), "{s}");
        }
    }
    #[test]
    fn limits() {
        assert!(evaluate(&"(".repeat(100), Angle::Degrees).is_err());
        assert!(evaluate(&"1".repeat(2049), Angle::Degrees).is_err());
        assert_eq!(calc("9999999999999999999999999999*100"), "1e30");
        assert!(evaluate("1e308*10", Angle::Degrees).is_err());
    }
    #[test]
    fn repeat_rhs() {
        let e = evaluate("2+3*4", Angle::Degrees).unwrap();
        let (op, rhs) = e.repeat.unwrap();
        assert_eq!(binary(op, e.value, rhs).unwrap().text(), "26");
    }
    #[test]
    fn radians() {
        assert!(
            (evaluate("sin(pi/2)", Angle::Radians)
                .unwrap()
                .value
                .as_f64()
                - 1.0)
                .abs()
                < 1e-14
        );
    }
    #[test]
    fn malformed_input_does_not_panic() {
        let alphabet: Vec<char> = "0123456789.+-*/^%!()eπ−×÷ sinta🦀".chars().collect();
        let mut seed = 20261023_u64;
        for length in 0..512 {
            let mut input = String::new();
            for _ in 0..length % 100 {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                input.push(alphabet[(seed >> 32) as usize % alphabet.len()]);
            }
            let _ = evaluate(&input, Angle::Degrees);
        }
    }
}
