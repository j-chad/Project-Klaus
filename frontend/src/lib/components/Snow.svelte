<!--Adapted from https://mannes.tech/svelte-snowfall-->

<script lang="ts">
    import { onMount } from 'svelte'

    interface Snowflake {
        scale: number
        x: number
        y: number
        opacity: number
        id: string
    }

    // a bunch of variables defining the snow and how it falls
    const SNOWFLAKES_COUNT = 200
    const SNOWFLAKE_MIN_SCALE = 0.1
    const SNOWFLAKE_MAX_SCALE = 0.7
    const MELTING_SPEED = 1.12
    const WIND_FORCE = 0.01
    const FALL_SPEED = 0.19
    const TARGET_FPS = 80

    const MS_BETWEEN_FRAMES = 1000 / TARGET_FPS

    function randomSnowflakeConfig(i: number): Snowflake {
        return {
            id: `snowflake-${i}`,
            scale: SNOWFLAKE_MIN_SCALE + Math.random() * SNOWFLAKE_MAX_SCALE,
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
    .snow {
        width: 10px;
        height: 10px;
        background: white;
        border-radius: 50%;
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
