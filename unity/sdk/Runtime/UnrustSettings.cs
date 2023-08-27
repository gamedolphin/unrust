using UnityEngine;

namespace unrust.runtime
{
    [CreateAssetMenu(fileName = "unrustGameSettings", menuName = "unrust/Create Settings", order = 1)]
    public class UnrustSettings : ScriptableObject
    {
        public string pathToProject;
    }
}
