pub mod parse;

use parse::{
    expressions::{Expr, ExprVariant, Fraction, GroupExpr, UnitExpr},
    tokenize::token::{Function, GroupingKind},
};

pub fn render(input: &str) -> String {
    let mut output = String::new();

    output.push_str("<math>");
    for expr in parse::Parser::new(input) {
        render_expr(expr, false, &mut output);
    }
    output.push_str("</math>");

    output
}

fn render_expr(expr: Expr, strip_parens: bool, output: &mut String) {
    match expr {
        Expr::Unit(unit) => render_unit(*unit, strip_parens, output),
        Expr::Fraction(fraction) => render_fraction(*fraction, output),
    }
}

fn render_unit(unit: UnitExpr, strip_parens: bool, output: &mut String) {
    let UnitExpr {
        variant,
        super_script,
        sub_script,
    } = unit;

    let tag = match (sub_script.is_some(), super_script.is_some()) {
        (true, true) => Some("msubsup"),
        (true, false) => Some("msub"),
        (false, true) => Some("msup"),
        (false, false) => None,
    };

    if let Some(tag) = tag {
        output.push_str(&format!("<{}>", tag));
    }

    render_variant(variant, strip_parens, output);

    if let Some(sub_script) = sub_script {
        render_expr(sub_script, true, output);
    }

    if let Some(super_script) = super_script {
        render_expr(super_script, true, output);
    }

    if let Some(tag) = tag {
        output.push_str(&format!("</{}>", tag));
    }
}

fn render_variant(variant: ExprVariant, strip_parens: bool, output: &mut String) {
    match variant {
        ExprVariant::Identifier(ident) => render_simple_tag("mi", &ident, output),
        ExprVariant::Operator(op) => render_operator(&op, output),
        ExprVariant::Num(num) => render_simple_tag("mn", &num, output),
        // TODO: escape text
        ExprVariant::Text(text) => render_simple_tag("mtext", &text, output),
        ExprVariant::Unary(function, expr) => render_unary(function, *expr, output),
        ExprVariant::Grouping(group) => render_group(group, strip_parens, output),
    }
}

fn render_group(group: GroupExpr, strip_parens: bool, output: &mut String) {
    output.push_str("<mrow>");

    let left = match group.left {
        GroupingKind::Paren => "(",
        GroupingKind::Bracket => "[",
        GroupingKind::Brace => "{",
    };

    let right = match group.right {
        GroupingKind::Paren => ")",
        GroupingKind::Bracket => "]",
        GroupingKind::Brace => "}",
    };

    let (left, right) =
        if strip_parens && group.left == group.right && group.left == GroupingKind::Paren {
            (None, None)
        } else {
            (Some(left), Some(right))
        };

    if let Some(left) = left {
        render_simple_tag("mo", left, output);
    }

    for expr in group.body {
        render_expr(expr, false, output);
    }

    if let Some(right) = right {
        render_simple_tag("mo", right, output);
    }

    output.push_str("</mrow>");
}

fn render_fraction(fraction: Fraction, output: &mut String) {
    output.push_str("<mfrac>");
    render_expr(fraction.numerator, true, output);
    render_expr(fraction.denominator, true, output);
    output.push_str("</mfrac>");
}

fn render_unary(function: Function, expr: Expr, output: &mut String) {
    todo!()
}

fn render_simple_tag(tag: &str, inner: &str, output: &mut String) {
    output.push_str(&format!("<{}>{}</{}>", tag, inner, tag));
}

fn render_operator(op: &str, output: &mut String) {
    render_simple_tag("mo", op, output);
}
