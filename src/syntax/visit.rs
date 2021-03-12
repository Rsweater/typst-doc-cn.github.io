//! Syntax tree traversal.

use super::*;

macro_rules! visit {
    ($(fn $name:ident($v:ident $(, $node:ident: &$ty:ty)?) $body:block)*) => {
        /// Traverses the syntax tree.
        pub trait Visit<'ast> {
            $(fn $name(&mut self $(, $node: &'ast $ty)?) {
                $name(self, $($node)?);
            })*

            /// Visit a definition of a binding.
            ///
            /// Bindings are, for example, left-hand side of let expressions,
            /// and key/value patterns in for loops.
            fn visit_binding(&mut self, _: &'ast Ident) {}

            /// Visit the entry into a scope.
            fn visit_enter(&mut self) {}

            /// Visit the exit from a scope.
            fn visit_exit(&mut self) {}
        }

        $(visit! {
            @$(concat!("Walk a node of type [`", stringify!($ty), "`]."), )?
            pub fn $name<'ast, V>(
                #[allow(unused)] $v: &mut V
                $(, #[allow(unused)] $node: &'ast $ty)?
            )
            where
                V: Visit<'ast> + ?Sized
            $body
        })*
    };

    (@$doc:expr, $($tts:tt)*) => {
        #[doc = $doc]
        $($tts)*
    };
}

visit! {
    fn visit_tree(v, node: &Tree) {
        for node in node {
            v.visit_node(&node);
        }
    }

    fn visit_node(v, node: &Node) {
        match node {
            Node::Strong => {}
            Node::Emph => {}
            Node::Space => {}
            Node::Linebreak => {}
            Node::Parbreak => {}
            Node::Text(_) => {}
            Node::Heading(n) => v.visit_tree(&n.contents),
            Node::Raw(_) => {}
            Node::Expr(expr) => v.visit_expr(expr),
        }
    }

    fn visit_expr(v, node: &Expr) {
        match node {
            Expr::Lit(_) => {}
            Expr::Ident(_) => {}
            Expr::Array(e) => v.visit_array(e),
            Expr::Dict(e) => v.visit_dict(e),
            Expr::Template(e) => v.visit_template(e),
            Expr::Group(e) => v.visit_group(e),
            Expr::Block(e) => v.visit_block(e),
            Expr::Unary(e) => v.visit_unary(e),
            Expr::Binary(e) => v.visit_binary(e),
            Expr::Call(e) => v.visit_call(e),
            Expr::Closure(e) => v.visit_closure(e),
            Expr::Let(e) => v.visit_let(e),
            Expr::If(e) => v.visit_if(e),
            Expr::While(e) => v.visit_while(e),
            Expr::For(e) => v.visit_for(e),
        }
    }

    fn visit_array(v, node: &ExprArray) {
        for expr in &node.items {
            v.visit_expr(&expr);
        }
    }

    fn visit_dict(v, node: &ExprDict) {
        for named in &node.items {
            v.visit_expr(&named.expr);
        }
    }

    fn visit_template(v, node: &ExprTemplate) {
        v.visit_enter();
        v.visit_tree(&node.tree);
        v.visit_exit();
    }

    fn visit_group(v, node: &ExprGroup) {
        v.visit_expr(&node.expr);
    }

    fn visit_block(v, node: &ExprBlock) {
        if node.scoping {
            v.visit_enter();
        }
        for expr in &node.exprs {
            v.visit_expr(&expr);
        }
        if node.scoping {
            v.visit_exit();
        }
    }

    fn visit_binary(v, node: &ExprBinary) {
        v.visit_expr(&node.lhs);
        v.visit_expr(&node.rhs);
    }

    fn visit_unary(v, node: &ExprUnary) {
        v.visit_expr(&node.expr);
    }

    fn visit_call(v, node: &ExprCall) {
        v.visit_expr(&node.callee);
        v.visit_args(&node.args);
    }

    fn visit_closure(v, node: &ExprClosure) {
        for param in node.params.iter() {
            v.visit_binding(param);
        }
        v.visit_expr(&node.body);
    }

    fn visit_args(v, node: &ExprArgs) {
        for arg in &node.items {
            v.visit_arg(arg);
        }
    }

    fn visit_arg(v, node: &ExprArg) {
        match node {
            ExprArg::Pos(expr) => v.visit_expr(&expr),
            ExprArg::Named(named) => v.visit_expr(&named.expr),
        }
    }

    fn visit_let(v, node: &ExprLet) {
        v.visit_binding(&node.binding);
        if let Some(init) = &node.init {
            v.visit_expr(&init);
        }
    }

    fn visit_if(v, node: &ExprIf) {
        v.visit_expr(&node.condition);
        v.visit_expr(&node.if_body);
        if let Some(body) = &node.else_body {
            v.visit_expr(&body);
        }
    }

    fn visit_while(v, node: &ExprWhile) {
        v.visit_expr(&node.condition);
        v.visit_expr(&node.body);
    }

    fn visit_for(v, node: &ExprFor) {
        match &node.pattern {
            ForPattern::Value(value) => v.visit_binding(value),
            ForPattern::KeyValue(key, value) => {
                v.visit_binding(key);
                v.visit_binding(value);
            }
        }
        v.visit_expr(&node.iter);
        v.visit_expr(&node.body);
    }
}