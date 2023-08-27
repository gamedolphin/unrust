using UnityEditor;
using UnityEngine;
using System;
using System.IO;
using System.Linq;
using System.Collections.Generic;
using System.Diagnostics;
using unrust.runtime;

namespace unrust.editor
{
    [InitializeOnLoad]
    public static class EnsureFolders
    {
        static EnsureFolders()
        {
            Build.EnsureFolders();
            var baseFolder = Path.GetFullPath("Packages/com.wavefunk.unrust/template~");
            CopyFolderAndRenameTemplates(baseFolder, Path.Combine(Application.dataPath, "../game"));

            var destinationAsmdef = Path.Combine(Application.dataPath, "unrust/UnrustGame.asmdef");
            if (!File.Exists(destinationAsmdef))
            {
                File.Copy(Path.Combine(baseFolder, "UnrustGame.asmdef"), destinationAsmdef);
            }

            var destinationSettings = Path.Combine(Application.dataPath, "unrust/unrustGameSettings.asset");
            if (!File.Exists(destinationSettings))
            {
                File.Copy(Path.Combine(baseFolder, "unrustGameSettings.asset"), destinationSettings);
            }
        }

        public static void CopyFolderAndRenameTemplates(string sourcePath, string destinationPath)
        {
            if (!Directory.Exists(sourcePath))
            {
                throw new DirectoryNotFoundException($"Source directory does not exist or could not be found: {sourcePath}");
            }

            // If the destination directory doesn't exist, create it.
            if (!Directory.Exists(destinationPath))
            {
                Directory.CreateDirectory(destinationPath);
            }
            else
            {
                // do not copy if destination exists
                return;
            }

            // Get the files in the source folder and copy them to the destination folder.
            FileInfo[] files = new DirectoryInfo(sourcePath).GetFiles();
            foreach (FileInfo file in files)
            {
                string destFileName = file.Name;

                if (file.Extension.Equals(".template", StringComparison.OrdinalIgnoreCase))
                {
                    destFileName = Path.GetFileNameWithoutExtension(file.Name);
                }
                else
                {
                    continue;
                }

                string destFile = Path.Combine(destinationPath, destFileName);
                file.CopyTo(destFile, true);
            }

            // Get the subfolders in the source folder and copy them to the destination folder.
            DirectoryInfo[] subFolders = new DirectoryInfo(sourcePath).GetDirectories();
            foreach (DirectoryInfo subFolder in subFolders)
            {
                string destDir = Path.Combine(destinationPath, subFolder.Name);
                CopyFolderAndRenameTemplates(subFolder.FullName, destDir);
            }
        }
    }

    public static class Build
    {
        [MenuItem("unrust/Recompile")]
        static void Recompile()
        {
            EnsureFolders();
            BuildGame();
        }

        internal static void EnsureFolders()
        {
            Directory.CreateDirectory(Path.GetFullPath(Path.Combine(Application.dataPath, "unrust")));
        }

        private static void BuildGame()
        {
            try
            {
                var asset = UnrustSettingsEditorFinder.Get();
                UnityEngine.Debug.Log($"Found project path: {asset.pathToProject}");
                var directory = Path.GetDirectoryName(Path.GetFullPath(Path.Combine(Application.dataPath, asset.pathToProject)));
                var outputDir = Path.GetFullPath(Path.Combine(Application.dataPath, "unrust"));

                EditorUtility.DisplayProgressBar("Cargo compile", "Compiling release build...", 0.5f);
                Compile(directory, outputDir);
                EditorUtility.ClearProgressBar();
                AssetDatabase.Refresh(ImportAssetOptions.ForceUpdate);
            }
            catch (Exception ex)
            {
                UnityEngine.Debug.LogError(ex);
                EditorUtility.ClearProgressBar();
                return;
            }
        }

        private static void Compile(string sourceDir, string outputDir)
        {
            var output = new List<String>();
            try
            {
                using (Process builder = new Process())
                {
                    builder.StartInfo.WorkingDirectory = sourceDir;
                    builder.StartInfo.UseShellExecute = false;
                    builder.StartInfo.FileName = "cargo";
                    var stmt = $"+nightly build --manifest-path {sourceDir}/Cargo.toml --out-dir {outputDir} -Z unstable-options";
                    UnityEngine.Debug.Log($"cargo {stmt}");
                    builder.StartInfo.Arguments = stmt;
                    builder.StartInfo.RedirectStandardError = true;
                    builder.StartInfo.RedirectStandardOutput = true;
                    builder.StartInfo.CreateNoWindow = true;
                    builder.OutputDataReceived += (s, e) => output.Add(e.Data);
                    builder.ErrorDataReceived += (s, e) => output.Add(e.Data);
                    builder.Start();
                    builder.BeginOutputReadLine();
                    builder.BeginErrorReadLine();
                    builder.WaitForExit();
                    UnityEngine.Debug.Log("Build done!");
                }
            }
            catch (Exception ex)
            {
                output.Add($"{ex}");
                UnityEngine.Debug.Log(String.Join('\n', output));
                return;
            }
            finally
            {
                UnityEngine.Debug.Log(String.Join('\n', output));
            }
        }
    }
}
