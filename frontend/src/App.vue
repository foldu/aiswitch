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
        <ul>
            <li><a :href="`${baseUrl}/doc`">API Documentation</a></li>
        </ul>
    </header>
    <main>
        <div v-if="!active">
            <p>Loading runners</p>
        </div>
        <div v-else>
            <p>Currently running {{ active.name }}</p>
            <p v-if="active.model">With model {{ active.model }}</p>
            <form @submit.prevent="changeRunner">
                <label for="runner">Runner</label>
                <select id="runner" v-model="selectedRunnerKey">
                    <option v-for="(_, name) in runners" :key="name" :value="name">
                        {{ name }}
                    </option>
                </select>
                <label v-if="selectedRunnerKey && runners[selectedRunnerKey].provides" for="model"
                    >Model</label
                >
                <select
                    v-if="selectedRunnerKey && runners[selectedRunnerKey].provides"
                    v-model="selectedModelKey"
                >
                    <option
                        v-for="model in runners[selectedRunnerKey].provides"
                        :key="model"
                        :value="model"
                    >
                        {{ model }}
                    </option>
                </select>
                <button type="submit">Change runner</button>
            </form>
        </div>
    </main>
</template>

<style scoped></style>
