<script setup lang="ts">
import { onMounted, ref, watch, type Ref } from "vue";
import createClient from "openapi-fetch";
import type { paths, components } from "./api.ts";

const baseUrl = import.meta.env.PROD ? "" : "http://localhost:4546";

const client = createClient<paths>({ baseUrl: baseUrl });
type Runner = components["schemas"]["Runner"];

type ActiveRunner = components["schemas"]["ActiveRunner"];

const runners: Ref<Record<string, Runner>> = ref({});
const active: Ref<ActiveRunner | null> = ref(null);
const selectedRunnerKey: Ref<string | null> = ref(null);
const selectedModelKey: Ref<string | null> = ref(null);

watch(selectedRunnerKey, (newRunner) => {
    if (newRunner && runners.value) {
        if (runners.value[newRunner].provides) {
            selectedModelKey.value = runners.value[newRunner].provides[0];
        } else {
            selectedModelKey.value = null;
        }
    }
});

async function changeRunner() {
    if (selectedRunnerKey.value) {
        const { data, error } = await client.PUT("/api/runner", {
            body: { name: selectedRunnerKey.value, model: selectedModelKey.value },
        });
        // active runner gets set by SSE, so setting it here is redundant
        if (!data) {
            console.log(error);
        }
    }
}

async function fetchRunners() {
    const { data, error } = await client.GET("/api/runner");
    if (data) {
        selectedRunnerKey.value = data.active.name;
        selectedModelKey.value = data.active.model ? data.active.model : null;
        active.value = data.active;
        runners.value = data.runners;
    } else {
        console.log(error);
    }
}

onMounted(async () => {
    await fetchRunners();
    const src = new EventSource(`${baseUrl}/api/update`);
    src.onmessage = (event) => {
        const data = JSON.parse(event.data);
        active.value = data;
        selectedRunnerKey.value = data.name;
        selectedModelKey.value = data.model;
    };
});
</script>

<template>
    <header>
        <ul class="header-nav">
            <li class="header-nav-item">
                <a :href="`${baseUrl}/doc`" class="header-nav-link">API Documentation</a>
            </li>
            <li class="header-nav-item">
                <a href="https://github.com/foldu/aiswitch" class="header-nav-link">Repository</a>
            </li>
        </ul>
    </header>
    <main class="main-content">
        <div v-if="!active">
            <p class="loading-message">Loading runners</p>
        </div>
        <div v-else>
            <p class="runner-info">Currently running {{ active.name }}</p>
            <p v-if="active.model" class="model-info">With model {{ active.model }}</p>
            <form class="runner-form" @submit.prevent="changeRunner">
                <label class="form-label" for="runner">Runner</label>
                <select id="runner" v-model="selectedRunnerKey" class="form-select">
                    <option v-for="(_, name) in runners" :key="name" :value="name">
                        {{ name }}
                    </option>
                </select>
                <label
                    v-if="selectedRunnerKey && runners[selectedRunnerKey].provides"
                    class="form-label"
                    for="model"
                    >Model</label
                >
                <select
                    v-if="selectedRunnerKey && runners[selectedRunnerKey].provides"
                    v-model="selectedModelKey"
                    class="form-select"
                >
                    <option
                        v-for="model in runners[selectedRunnerKey].provides"
                        :key="model"
                        :value="model"
                    >
                        {{ model }}
                    </option>
                </select>
                <button type="submit" class="submit-button">Change runner</button>
            </form>
        </div>
    </main>
</template>
<style scoped>
header {
    background-color: #f0f0f0;
    padding: 1rem;
    text-align: center;
}

main {
    padding: 2rem;
    max-width: 600px;
    margin: 0 auto;
}

.header-nav {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    justify-content: center;
}

.header-nav-item {
    margin: 0 1rem;
}

.header-nav-link {
    text-decoration: none;
    color: #333;
    font-weight: bold;
}

.loading-message {
    font-style: italic;
    color: #777;
}

.runner-info {
    font-weight: bold;
    margin-bottom: 0.5rem;
}

.model-info {
    color: #555;
}

.runner-form {
    display: flex;
    flex-direction: column;
    margin-top: 1rem;
}

.form-label {
    margin-bottom: 0.5rem;
    font-weight: bold;
}

.form-select {
    padding: 0.5rem;
    margin-bottom: 1rem;
    border: 1px solid #ccc;
    border-radius: 4px;
}

.submit-button {
    background-color: #4caf50;
    color: white;
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
}

.submit-button:hover {
    background-color: #3e8e41;
}
</style>
