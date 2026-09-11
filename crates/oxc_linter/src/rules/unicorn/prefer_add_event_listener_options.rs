use oxc_ast::{
    AstKind,
    ast::{Argument, Expression, MemberExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn prefer_add_event_listener_options_diagnostic(span: Span, replacement: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Prefer `{replacement}` over a boolean `useCapture` argument."))
        .with_help(format!("Pass `{replacement}` as an options object."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferAddEventListenerOptions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefers an options object over a boolean as the third argument of `addEventListener()`.
    ///
    /// ### Why is this bad?
    ///
    /// The boolean form is a [flag argument](https://martinfowler.com/bliki/FlagArgument.html):
    /// a reader must know the signature to tell what `true` means. `{capture: true}` states the
    /// intent, and the options object also accepts `passive`, `once` and `signal`, which the
    /// boolean form cannot express.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// element.addEventListener('click', handleClick, true);
    /// element.addEventListener('scroll', handleScroll, false);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// element.addEventListener('click', handleClick, {capture: true});
    /// element.addEventListener('scroll', handleScroll, {capture: false});
    /// element.addEventListener('touchmove', handleScroll, {passive: true});
    /// ```
    PreferAddEventListenerOptions,
    unicorn,
    style,
    fix,
    version = "next",
    short_description = "Prefer an options object over a boolean in `.addEventListener()`.",
);

impl Rule for PreferAddEventListenerOptions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if call_expr.optional {
            return;
        }

        let Some(member_expr) = call_expr.callee.get_member_expr() else {
            return;
        };

        let MemberExpression::StaticMemberExpression(v) = member_expr else {
            return;
        };

        if v.property.name != "addEventListener" || v.optional {
            return;
        }

        let [_, _, third] = call_expr.arguments.as_slice() else {
            return;
        };
        if call_expr.arguments.iter().any(Argument::is_spread) {
            return;
        }

        let Some(Expression::BooleanLiteral(literal)) =
            third.as_expression().map(Expression::get_inner_expression)
        else {
            return;
        };
        let replacement = if literal.value { "{capture: true}" } else { "{capture: false}" };

        ctx.diagnostic_with_fix(
            prefer_add_event_listener_options_diagnostic(literal.span, replacement),
            |fixer| fixer.replace(literal.span, replacement),
        );
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"window.addEventListener("click", listener)"#,
        r#"window.addEventListener("click", listener, {capture: true})"#,
        r#"window.addEventListener("click", listener, {capture: false})"#,
        r#"window.addEventListener("click", listener, {passive: true})"#,
        r#"window.addEventListener("click", listener, {once: true})"#,
        r#"window.addEventListener("click", listener, {signal})"#,
        r#"window.addEventListener("click", listener, options)"#,
        r#"window.addEventListener("click", listener, capture)"#,
        r#"window.addEventListener("click", listener, Boolean(value))"#,
        r#"window.addEventListener("click", listener, condition ? true : false)"#,
        r#"window.addEventListener("click", listener, undefined)"#,
        r#"window.addEventListener("click", listener, null)"#,
        r#"window["addEventListener"]("click", listener, true)"#,
        r#"window?.addEventListener("click", listener, true)"#,
        r#"window.addEventListener?.("click", listener, true)"#,
        r#"window.addEventListener("click", ...arguments_, true)"#,
    ];

    let fail = vec![
        r#"window.addEventListener("click", listener, true)"#,
        r#"window.addEventListener("click", listener, false)"#,
        r#"window.addEventListener("click", listener, (true))"#,
        r#"window.addEventListener("click", () => {}, true)"#,
        r#"window.addEventListener("click", function () {}, false)"#,
        r#"document.body.addEventListener("click", listener, true)"#,
        r#"(window).addEventListener("click", listener, false)"#,
        r#"window.addEventListener("click", listener, /* useCapture */ true)"#,
        r#"window.addEventListener("click", listener, true /* useCapture */)"#,
        r#"window.addEventListener(
                "click",
                listener,
                true
            )"#,
    ];

    let fix = vec![
        (
            r#"window.addEventListener("click", listener, true)"#,
            r#"window.addEventListener("click", listener, {capture: true})"#,
        ),
        (
            r#"window.addEventListener("click", listener, false)"#,
            r#"window.addEventListener("click", listener, {capture: false})"#,
        ),
        (
            r#"window.addEventListener("click", listener, (true))"#,
            r#"window.addEventListener("click", listener, ({capture: true}))"#,
        ),
        (
            r#"window.addEventListener("click", listener, /* useCapture */ true)"#,
            r#"window.addEventListener("click", listener, /* useCapture */ {capture: true})"#,
        ),
    ];

    Tester::new(
        PreferAddEventListenerOptions::NAME,
        PreferAddEventListenerOptions::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
