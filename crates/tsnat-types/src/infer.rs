use std::collections::HashMap;
use crate::ty::{TypeArena, TypeId, Type, GenericType};

pub struct TypeInferencer<'a> {
    pub arena: &'a mut TypeArena,
}

impl<'a> TypeInferencer<'a> {
    pub fn new(arena: &'a mut TypeArena) -> Self {
        Self { arena }
    }

    /// Instantiate a generic type with a specific list of type arguments.
    /// E.g., resolving `Array<string>` from `Array<T>`.
    pub fn instantiate_generic(&mut self, target: TypeId, args: Vec<TypeId>) -> TypeId {
        // Here we build a map from TypeParam(name) -> arg
        // and then deeply clone the target, substituting the params.
        
        let target_ty = self.arena.get(target).clone();
        
        if let Type::Function(ref func) = target_ty {
            // Very simplified: Assuming the target function HAS type parameters defined somewhere.
            // Right now our FunctionType doesn't store type parameters! Wait, let's just 
            // do this generally for now and we can expand it.
            let _ = func;
        }

        // We wrap the instantiated version in a Generic node so the checker knows 
        // it came from an instantiation (useful for diagnostics).
        
        // Return a structural instantiation representation
        self.arena.alloc(Type::Generic(GenericType {
            target,
            args,
        }))
    }

    /// Deep recursive clone of a Type substituting parameters based on the map
    pub fn substitute(&mut self, ty_id: TypeId, substitutions: &HashMap<tsnat_common::interner::Symbol, TypeId>) -> TypeId {
        let ty = self.arena.get(ty_id).clone();
        match ty {
            Type::TypeParam(p) => {
                if let Some(&substituted) = substitutions.get(&p.name) {
                    substituted
                } else {
                    ty_id
                }
            }
            Type::Union(variants) => {
                let new_vars = variants.into_iter().map(|v| self.substitute(v, substitutions)).collect();
                self.arena.alloc(Type::Union(new_vars))
            }
            Type::Object(obj) => {
                let mut new_props = indexmap::IndexMap::new();
                for (name, prop) in obj.properties {
                    new_props.insert(name, crate::ty::PropertyType {
                        ty: self.substitute(prop.ty, substitutions),
                        optional: prop.optional,
                        readonly: prop.readonly,
                    });
                }
                self.arena.alloc(Type::Object(crate::ty::ObjectType { properties: new_props }))
            }
            Type::Function(mut func) => {
                for param in &mut func.params {
                    param.ty = self.substitute(param.ty, substitutions);
                }
                func.return_ty = self.substitute(func.return_ty, substitutions);
                self.arena.alloc(Type::Function(func))
            }
            Type::Generic(mut gx) => {
                for arg in &mut gx.args {
                    *arg = self.substitute(*arg, substitutions);
                }
                self.arena.alloc(Type::Generic(gx))
            }
            // For base cases and primitives, we just return the original ID
            _ => ty_id,
        }
    }

    /// Evaluates `T extends U ? X : Y`. Distributes over `T` if it is a Union.
    pub fn evaluate_conditional(&mut self, check_type: TypeId, extends_type: TypeId, true_type: TypeId, false_type: TypeId) -> TypeId {
        let ct = self.arena.get(check_type).clone();
        if let Type::Union(variants) = ct {
            // Distribute: (A | B) extends U ? X : Y -> (A extends U ? X : Y) | (B extends U ? X : Y)
            let mut eval_variants = Vec::new();
            for var in variants {
                eval_variants.push(self.evaluate_conditional(var, extends_type, true_type, false_type));
            }
            return self.arena.alloc(Type::Union(eval_variants));
        }

        // Lazy or concrete evaluation
        // If check_type is a TypeParam, we must defer execution because it's a generic parameter.
        if matches!(ct, Type::TypeParam(_)) {
            // Defer execution
            return self.arena.alloc(Type::Conditional(crate::ty::ConditionalType { check_type, extends_type, true_type, false_type }));
        }

        // Concrete check using assignability
        let checker = crate::assignability::AssignabilityChecker::new(self.arena);
        if checker.is_assignable(check_type, extends_type) {
            true_type
        } else {
            false_type
        }
    }
}


