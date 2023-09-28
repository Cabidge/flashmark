pub mod parse;

use std::fmt::{self, Display};

use parse::{
    expressions::{Expr, ExprVariant, Fraction, GroupExpr, UnitExpr},
    tokenize::Function,
};

pub fn render(input: &str) -> String {
    let mut output = String::new();

    output.push_str("<math>");
    render_row(parse::Parser::new(input), &mut output).expect("Writing to string should not fail");
    output.push_str("</math>");

    output
}

fn render_row(exprs: impl IntoIterator<Item = Expr>, output: &mut impl fmt::Write) -> fmt::Result {
    output.write_str("<mrow>")?;

    for expr in exprs {
        render_expr(expr, false, output)?;
    }

    output.write_str("</mrow>")
}

fn render_expr(expr: Expr, strip_parens: bool, output: &mut impl fmt::Write) -> fmt::Result {
    match expr {
        Expr::Unit(unit) => render_unit(*unit, strip_parens, output),
        Expr::Fraction(fraction) => render_fraction(*fraction, output),
    }
}

fn render_unit(unit: UnitExpr, strip_parens: bool, output: &mut impl fmt::Write) -> fmt::Result {
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
        write!(output, "<{}>", tag)?;
    }

    render_variant(variant, strip_parens, output)?;

    if let Some(sub_script) = sub_script {
        render_variant(sub_script, true, output)?;
    }

    if let Some(super_script) = super_script {
        render_expr(super_script, true, output)?;
    }

    if let Some(tag) = tag {
        write!(output, "</{}>", tag)?;
    }

    Ok(())
}

fn render_variant(
    variant: ExprVariant,
    strip_parens: bool,
    output: &mut impl fmt::Write,
) -> fmt::Result {
    match variant {
        ExprVariant::Identifier(ident) => render_simple_tag("mi", &ident, output),
        ExprVariant::Operator(op) => render_operator(&op, output),
        ExprVariant::Num(num) => render_simple_tag("mn", &num, output),
        ExprVariant::Text(text) => render_simple_tag("mtext", &text, output),
        ExprVariant::Unary(function, expr) => render_unary(function, *expr, output),
        ExprVariant::Grouping(group) => render_group(group, strip_parens, output),
    }
}

fn render_group(group: GroupExpr, strip_parens: bool, output: &mut impl fmt::Write) -> fmt::Result {
    if strip_parens && group.has_parens() {
        return render_row(group.body, output);
    }

    output.write_str("<mrow>")?;

    let left = group.left.into_left_char();
    render_simple_tag("mo", left, output)?;

    for expr in group.body {
        render_expr(expr, false, output)?;
    }

    let right = group.right.into_right_char();
    render_simple_tag("mo", right, output)?;

    output.write_str("</mrow>")?;

    Ok(())
}

fn render_fraction(fraction: Fraction, output: &mut impl fmt::Write) -> fmt::Result {
    output.write_str("<mfrac>")?;
    render_expr(fraction.numerator, true, output)?;
    render_expr(fraction.denominator, true, output)?;
    output.write_str("</mfrac>")?;

    Ok(())
}

fn render_unary(function: Function, expr: Expr, output: &mut impl fmt::Write) -> fmt::Result {
    match function {
        Function::Sqrt => {
            output.write_str("<msqrt>")?;
            render_expr(expr, true, output)?;
            output.write_str("</msqrt>")?;
        }
        Function::Sin => {
            render_simple_tag("mi", "sin", output)?;
            render_expr(expr, false, output)?;
        }
        Function::Cos => {
            render_simple_tag("mi", "cos", output)?;
            render_expr(expr, false, output)?;
        }
        Function::Tan => {
            render_simple_tag("mi", "tan", output)?;
            render_expr(expr, false, output)?;
        }
    }

    Ok(())
}

fn render_simple_tag(tag: &str, inner: impl Display, output: &mut impl fmt::Write) -> fmt::Result {
    write!(output, "<{}>{}</{}>", tag, inner, tag)
}

fn render_operator(op: impl Display, output: &mut impl fmt::Write) -> fmt::Result {
    render_simple_tag("mo", op, output)
}
