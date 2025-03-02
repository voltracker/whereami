<template>
  <div ref="globeDiv"></div>
</template>

<script>
import Globe from "globe.gl";
import { ref, onMounted } from "vue";

export default {
  setup() {
    const globeDiv = ref("globeDiv");

    onMounted(() => {
      const myGlobe = Globe();
      myGlobe(globeDiv).globeImageUrl(
        "//unpkg.com/three-globe/example/img/earth-night.jpg",
      );
    });

    return {
      globeDiv,
    };
  },
};
</script>
