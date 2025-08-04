<!--Adapted from https://mannes.tech/svelte-snowfall-->

<script lang="ts">
    import { onMount } from 'svelte'

    // a bunch of variables defining the snow and how it falls
    const SNOWFLAKES_COUNT = 70
    const SNOWFLAKE_MIN_SCALE = 0.1
    const MELTING_SPEED = 1.12
    const WIND_FORCE = 0.01
    const FALL_SPEED = 0.09
    const TARGET_FPS = 80

    const MS_BETWEEN_FRAMES = 1000 / TARGET_FPS

    interface Snowflake {
        scale: number
        x: number
        y: number
        opacity: number
        id: string
    }

    function randomSnowflakeConfig(i: number): Snowflake {
        return {
            id: `snowflake-${i}`,
            scale: SNOWFLAKE_MIN_SCALE + Math.random() * (1 - SNOWFLAKE_MIN_SCALE),
            x: -20 + Math.random() * 120,
            y: -100 + Math.random() * 200,
            opacity: 0.999,
        }
    }

    let snowflakes = Array.from({ length: SNOWFLAKES_COUNT }, (_, i) => randomSnowflakeConfig(i));

    onMount(() => {
        let frame: number, lastTime = performance.now()

        function loop(timestamp: DOMHighResTimeStamp) {
            frame = requestAnimationFrame(loop)

            const elapsed = timestamp - lastTime
            lastTime = timestamp

            let framesCompleted = elapsed / MS_BETWEEN_FRAMES

            snowflakes = snowflakes.map(flake => {
                if (flake.y >= 100) {
                    flake.opacity = Math.pow(flake.opacity, MELTING_SPEED)
                } else {
                    flake.y += FALL_SPEED * flake.scale * framesCompleted
                    flake.x += WIND_FORCE * flake.scale * framesCompleted
                }
                if (flake.opacity <= 0.02) {
                    flake.y = -20
                    flake.x = -20 + Math.random() * 120
                    flake.opacity = 0.999
                }
                return flake
            })
        }

        frame = requestAnimationFrame(loop);

        return () => cancelAnimationFrame(frame)
    })
</script>

<style>
    :global(body) {
        min-height: 100%;
    }

    :global(html) {
        height: 100%;
    }

    .snow {
        width: 10px;
        height: 10px;
        background: white;
        border-radius: 50%;
        box-shadow: 0 0 5px rgba(255, 255, 255, 0.5);
        position: absolute;
        z-index: 1000;
        overflow: hidden;
    }

    .snowframe {
        pointer-events: none;
        position: absolute;
        top: 0;
        right: 0;
        bottom: 0;
        left: 0;
        overflow: hidden;
    }
</style>

<div class="snowframe" aria-hidden="true">
    {#each snowflakes as flake (flake.id)}
        <div
                class="snow"
                style={`opacity: ${flake.opacity}; transform: scale(${flake.scale}); left: ${flake.x}%; top: calc(${flake.y}% - ${flake.scale}rem)`}>
        </div>
    {/each}
</div>
