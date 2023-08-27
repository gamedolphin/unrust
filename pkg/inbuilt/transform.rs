use bevy::prelude::*;
use genco::prelude::*;
use unrust_proc_macro::unity_authoring;

#[unity_authoring]
pub struct UnityTransform {
    pub mat: [f32; 16],
}

#[allow(non_snake_case)]
pub fn UnityTransform_ingest_component(
    entity: &mut bevy::ecs::world::EntityMut,
    val: &UnityTransform,
) {
    let bundle =
        TransformBundle::from_transform(Transform::from_matrix(Mat4::from_cols_array(&val.mat)));
    entity.insert(bundle);
}

impl From<UnityTransform> for Transform {
    fn from(value: UnityTransform) -> Self {
        Transform::from_matrix(Mat4::from_cols_array(&value.mat))
    }
}

impl From<Transform> for UnityTransform {
    fn from(value: Transform) -> Self {
        UnityTransform {
            mat: value.compute_matrix().to_cols_array(),
        }
    }
}

impl From<&Transform> for UnityTransform {
    fn from(value: &Transform) -> Self {
        UnityTransform {
            mat: value.compute_matrix().to_cols_array(),
        }
    }
}

impl From<&GlobalTransform> for UnityTransform {
    fn from(value: &GlobalTransform) -> Self {
        UnityTransform {
            mat: value.compute_matrix().to_cols_array(),
        }
    }
}

#[allow(non_snake_case)]
pub fn UnityTransform_CSHARP_TOKEN() -> csharp::Tokens {
    quote! {
        [StructLayout(LayoutKind.Sequential)]
        public unsafe struct UnityTransform
        {
            public fixed float matrix[16];

            public static implicit operator Unity.Transforms.LocalTransform(UnityTransform val) => Unity.Transforms.LocalTransform.FromMatrix(new Unity.Mathematics.float4x4(
                val.matrix[0], val.matrix[4], val.matrix[8], val.matrix[12],
                val.matrix[1], val.matrix[5], val.matrix[9], val.matrix[13],
                val.matrix[2], val.matrix[6], val.matrix[10], val.matrix[14],
                val.matrix[3], val.matrix[7], val.matrix[11], val.matrix[15]
            ));

            public static implicit operator Unity.Transforms.LocalToWorld(UnityTransform val) => new Unity.Transforms.LocalToWorld
            {
                Value = new Unity.Mathematics.float4x4(
                    val.matrix[0], val.matrix[4], val.matrix[8], val.matrix[12],
                    val.matrix[1], val.matrix[5], val.matrix[9], val.matrix[13],
                    val.matrix[2], val.matrix[6], val.matrix[10], val.matrix[14],
                    val.matrix[3], val.matrix[7], val.matrix[11], val.matrix[15]
                )
            };


            public static implicit operator UnityTransform(Unity.Transforms.LocalTransform output)
            {
                var transform = new UnityTransform();
                var val = output.ToMatrix();
                transform.matrix[0] = val.c0[0];
                transform.matrix[1] = val.c0[1];
                transform.matrix[2] = val.c0[2];
                transform.matrix[3] = val.c0[3];
                transform.matrix[4] = val.c1[0];
                transform.matrix[5] = val.c1[1];
                transform.matrix[6] = val.c1[2];
                transform.matrix[7] = val.c1[3];
                transform.matrix[8] = val.c2[0];
                transform.matrix[9] = val.c2[1];
                transform.matrix[10] = val.c2[2];
                transform.matrix[11] = val.c2[3];
                transform.matrix[12] = val.c3[0];
                transform.matrix[13] = val.c3[1];
                transform.matrix[14] = val.c3[2];
                transform.matrix[15] = val.c3[3];

                return transform;
            }
        }
    }
}
