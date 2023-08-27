fn main() -> anyhow::Result<()> {
    inbuilt::write_csharp_inbuilt("../../unity/sdk/Runtime/InbuiltGenerated.cs")?;

    Ok(())
}
