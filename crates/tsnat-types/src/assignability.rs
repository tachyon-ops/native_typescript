use crate::ty::{TypeArena, TypeId, Type, TYPE_NEVER, TYPE_ANY, TYPE_UNKNOWN};

pub struct AssignabilityChecker<'a> {
    arena: &'a TypeArena,
}

impl<'a> AssignabilityChecker<'a> {
    pub fn new(arena: &'a TypeArena) -> Self {
        Self { arena }
    }

    /// Check if `source` is assignable to `target` (i.e. source <: target).
    pub fn is_assignable(&self, source: TypeId, target: TypeId) -> bool {
        // Identity
        if source == target {
            return true;
        }

        let src_ty = self.arena.get(source);
        let tgt_ty = self.arena.get(target);

        // never is assignable to everywhere
        if matches!(src_ty, Type::Never) {
            return true;
        }

        // any is assignable to everywhere (and everywhere to any)
        if matches!(src_ty, Type::Any) || matches!(tgt_ty, Type::Any) {
            return true;
        }

        // unknown is a top type
        if matches!(tgt_ty, Type::Unknown) {
            return true;
        }

        match (src_ty, tgt_ty) {
            // Primitives
            (Type::LiteralNumber(_), Type::Number) => true,
            (Type::LiteralString(_), Type::String) => true,
            (Type::LiteralBool(_), Type::Boolean) => true,

            // Target is Union (source <: A | B if source <: A or source <: B)
            (_, Type::Union(targets)) => {
                targets.iter().any(|&t| self.is_assignable(source, t))
            }

            // Source is Union (A | B <: target if A <: target and B <: target)
            (Type::Union(sources), _) => {
                sources.iter().all(|&s| self.is_assignable(s, target))
            }

            // Target is Intersection (source <: A & B if source <: A and source <: B)
            (_, Type::Intersection(targets)) => {
                targets.iter().all(|&t| self.is_assignable(source, t))
            }

            // Source is Intersection (A & B <: target if A <: target or B <: target)
            (Type::Intersection(sources), _) => {
                sources.iter().any(|&s| self.is_assignable(s, target))
            }

            (Type::Object(src_obj), Type::Object(tgt_obj)) => {
                // Structural matching
                // target source must have all required properties of target
                for (key, tgt_prop) in tgt_obj.properties.iter() {
                    if let Some(src_prop) = src_obj.properties.get(key) {
                        if !self.is_assignable(src_prop.ty, tgt_prop.ty) {
                            return false;
                        }
                    } else if !tgt_prop.optional {
                        // missing required property
                        return false;
                    }
                }
                true
            }

            // Function assignability
            (Type::Function(src_fn), Type::Function(tgt_fn)) => {
                // Contravariant parameters: target params must be assignable to source params
                // Covariant return: source return must be assignable to target return
                if !self.is_assignable(src_fn.return_ty, tgt_fn.return_ty) {
                    return false;
                }

                // Actually TS allows fewer parameters in the source function, which means:
                // `(a: string) => void` is assignable to `(a: string, b: number) => void`
                if src_fn.params.len() > tgt_fn.params.len() {
                    // But if source requires MORE parameters, it's an error (unless optional)
                    // We'll enforce simple length check for now based on strictness.
                    return false;
                }

                for (i, src_param) in src_fn.params.iter().enumerate() {
                    let tgt_param = &tgt_fn.params[i];
                    if !self.is_assignable(tgt_param.ty, src_param.ty) {
                        return false;
                    }
                }

                true
            }

            // TypeParams
            (Type::TypeParam(_), Type::TypeParam(_)) => {
                // strict identity check for uninstantiated params usually
                // In a real checker, this involves checking environments. We default to false if not identical.
                false // Identity is handled at the start
            }
            (_src, Type::TypeParam(_)) => {
                // We cannot assign TO a type parameter unless it's the exact same type parameter (handled by identity)
                // or if it has a lower bound (not implemented). But we cannot assign to it based on its upper bound constraint!
                false
            }
            (Type::TypeParam(p), _tgt) => {
                if let Some(constraint) = p.constraint {
                    self.is_assignable(constraint, target)
                } else {
                    false
                }
            }

            // Generics bounds checking (lazy deep structural evaluation)
            // Note: For Phase 3B, a generic like `Array<string>` <: `Array<string>`.
            // The structural extraction handles the deeper logic.
            (Type::Generic(src_gen), Type::Generic(tgt_gen)) => {
                if src_gen.target != tgt_gen.target {
                    return false;
                }
                if src_gen.args.len() != tgt_gen.args.len() {
                    return false;
                }
                // Check argument variance (we will assume invariant for simplicity here)
                for (i, src_arg) in src_gen.args.iter().enumerate() {
                    let tgt_arg = tgt_gen.args[i];
                    if !self.is_assignable(*src_arg, tgt_arg) && !self.is_assignable(tgt_arg, *src_arg) {
                        return false; // invariant check
                    }
                }
                true
            }

            _ => false,
        }
    }
}

