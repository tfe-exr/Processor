<script lang="ts">

</script>

<div class="root">
    <slot />
</div>

<style lang="scss">
    @import "../../../conf/spacing.scss";
    @import "../../../conf/surface.scss";

    .root {
        padding: $spacing__body;
        background: $surface__body;
        flex: 1;
        display: flex;
        flex-direction: column;
        gap: $spacing__body;
    }
</style>