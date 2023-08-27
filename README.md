
![Crates.io](https://img.shields.io/crates/v/unrust?link=https%3A%2F%2Fcrates.io%2Fcrates%2Funrust)
# Unrust - Rust + Bevy + Unity to make games!
![Peek 2023-08-27 18-20](https://github.com/gamedolphin/unrust/assets/7590634/17b9e2a9-e0a1-4891-82a3-133fa32e86ec)

You want to use Rust + Bevy to make your games. But you miss a good editor to go with it? 

Well this is a zero level attempt to use Unity as the editor. You setup your scenes and prefabs using unity. 

And then everything else is done in your rust codebase. GG EZ.

(only works on Unity 2022, LINUX . windows and mac soon)

Also requires entities + entities graphics packages. Maps unity entities to bevy entities. Does not work with gameobjects yet. 

This is VERY VERY VERY experimental. There's a ton of work to add more component syncronization and improved devx.

I'll be adding docs soon, but if you do not mind wading into some undocumented rust + c# code, you're welcome to contribute! 

## Documentation

TODO!


## Installation

Download the tarball from releases on the left, and it to a unity project. 
It would/should create a `game` folder besides the Asset folder with a basic project. 
Hit unrust -> Compile in the unity project menu, and press play. 

Requires rust nightly (to properly place the output library path and for codegen)

## Usage/Examples

Defining a new struct with `unity_authoring` attribute in the `types.rs` file would also generate a corresponding Authoring component in the unity Assets/unrust folder. You can then attach this to an entity to have it be created in the bevy world too on play.

```rust
#[unity_authoring]
pub struct SampleComponent {
    pub val: i32,
}
```

```csharp
using System.Runtime.InteropServices;
using Unity.Entities;
using UnityEngine;

namespace unrust.userland
{
    [StructLayout(LayoutKind.Sequential)]
    public struct SampleComponent : IComponentData
    {
        public float speed;
    }

    public class SampleComponentAuthoring : MonoBehaviour
    {
        public float speed;

        class Baker : Baker<SampleComponentAuthoring>
        {
            public override void Bake(SampleComponentAuthoring authoring)
            {
                var entity = GetEntity(TransformUsageFlags.Dynamic);
                AddComponent(entity, new SampleComponent
                    {
                        speed = authoring.speed,
                    });
            }
        }
    }
}

```
## TODOS

- Additional platform support

- Add more built-in types. (Only transform and parents are synced right now)

- Handle inputs

- Handle camera / other non-ecs gameobjects.

- Wasm?!


## License

[MIT](https://choosealicense.com/licenses/mit/)

