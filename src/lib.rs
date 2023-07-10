use swc_core::{
    common::{util::take::Take, SourceMap, DUMMY_SP},
    ecma::{
        ast::{
            EsVersion, Expr, ExprStmt, Ident, ImportSpecifier, ModuleDecl, ModuleItem, Program,
            Script, Stmt, TaggedTpl,
        },
        codegen::{
            text_writer::{self, JsWriter, WriteJs},
            Config, Emitter,
        },
        visit::*,
    },
    plugin::{plugin_transform, proxies::TransformPluginProgramMetadata},
};

struct ImportVisitor {
    pub import_ident: Option<Ident>,
}

impl VisitMut for ImportVisitor {
    fn visit_mut_module_item(&mut self, node: &mut ModuleItem) {
        match node {
            ModuleItem::ModuleDecl(ModuleDecl::Import(import)) => {
                if import.src.value == *"swc-plugin-preeval/preeval" {
                    let specifiers = &import.specifiers;
                    if specifiers.len() == 1 {
                        let specifier = &specifiers[0];

                        if let ImportSpecifier::Named(named) = specifier {
                            let import_ident = named
                                .imported
                                .as_ref()
                                .map(|named| match named {
                                    swc_ecma_ast::ModuleExportName::Ident(ident) => ident.clone(),
                                    _ => panic!("not ident"),
                                })
                                .unwrap_or_else(|| named.local.clone());

                            self.import_ident = Some(import_ident.clone());

                            // It is a dummy import, remove it
                            node.take();
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

struct CompileVisitor {
    pub import_ident: Ident,
}

impl VisitMut for CompileVisitor {
    fn visit_mut_expr(&mut self, e: &mut Expr) {
        e.visit_mut_children_with(self);

        match e {
            Expr::TaggedTpl(TaggedTpl { tag, .. }) => match &**tag {
                Expr::Ident(ident) => {
                    if ident.sym == self.import_ident.sym {
                        let mut buf = vec![];

                        {
                            let mut wr =
                                Box::new(JsWriter::new(Default::default(), "\n", &mut buf, None))
                                    as Box<dyn WriteJs>;

                            wr = Box::new(text_writer::omit_trailing_semi(wr));

                            let mut emitter = Emitter {
                                cfg: Config {
                                    minify: true,
                                    target: EsVersion::Es5,
                                    ascii_only: true,
                                    ..Default::default()
                                },
                                cm: std::sync::Arc::new(SourceMap::default()),
                                wr,
                                comments: None,
                            };

                            let mut s = Script::dummy();

                            s.body.push(Stmt::Expr(ExprStmt {
                                span: DUMMY_SP,
                                expr: Box::new(e.take()),
                            }));

                            emitter.emit_script(&s).expect("failed to emit script");
                        }

                        let s = String::from_utf8(buf).expect("invalid utf8 character detected");
                        let s = s
                            .strip_prefix(&format!("{}`", self.import_ident.sym))
                            .unwrap();
                        let s = s.strip_suffix("`;").unwrap();

                        let mut context = boa_engine::Context::default();

                        let result = context
                            .eval(boa_engine::Source::from_bytes(&s))
                            .expect("failed to eval script");

                        let result = if result.is_string() {
                            format!("\"{}\"",result.as_string().unwrap().to_std_string_escaped())
                        } else {
                            result
                                .to_string(&mut context)
                                .unwrap()
                                .to_std_string_escaped()
                        };

                        let cm: swc_core::common::sync::Lrc<swc_core::common::SourceMap> =
                            Default::default();
                        let fm =
                            cm.new_source_file(swc_core::common::FileName::Anon, result.into());

                        let lexer_input = swc_core::ecma::parser::StringInput::from(&*fm);

                        let lexer = swc_core::ecma::parser::lexer::Lexer::new(
                            swc_core::ecma::parser::Syntax::Es(Default::default()),
                            Default::default(),
                            lexer_input,
                            None,
                        );

                        let mut ecma_parser = swc_core::ecma::parser::Parser::new_from(lexer);

                        match ecma_parser.parse_expr() {
                            Ok(expr) => {
                                *e = *expr;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        };
    }
}

#[plugin_transform]
pub fn process(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    let mut program = program;
    let mut import_visitor = ImportVisitor { import_ident: None };
    program.visit_mut_with(&mut import_visitor);

    if let Some(import_ident) = import_visitor.import_ident {
        let mut compile_visitor = CompileVisitor { import_ident };
        program.visit_mut_with(&mut compile_visitor);
    }

    program
}
