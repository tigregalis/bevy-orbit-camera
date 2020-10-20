# Orbit camera test in bevy

The example follows the cube:

1. on `OrbitCamera` we store the target which is the cube's entity
2. we attach this `OrbitCamera` component to the camera entity
3. on the cube entity we attach the marker `OrbitCameraTarget` component

Use mousewheel to zoom

Press and hold the mousewheel down, and drag to rotate