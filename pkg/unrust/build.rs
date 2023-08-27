fn main() -> anyhow::Result<()> {
    if std::env::var_os("UNRUST_DEV").is_none() {
        return Ok(());
    }

    inbuilt::write_csharp_inbuilt("../../unity/sdk/Runtime/InbuiltGenerated.cs")?;
    Ok(())
}
