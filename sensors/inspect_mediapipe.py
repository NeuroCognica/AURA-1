import mediapipe
import pkgutil

print("mediapipe file:", mediapipe.__file__)
print("mediapipe version:", getattr(mediapipe, "__version__", "unknown"))

print("\nTop-level attributes:")
print([a for a in dir(mediapipe) if not a.startswith("_")])

print("\nSearching for face-related modules...")
for m in pkgutil.walk_packages(mediapipe.__path__, mediapipe.__name__ + "."):
    if "face" in m.name.lower():
        print("FOUND:", m.name)
