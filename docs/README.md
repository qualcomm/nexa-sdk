# Getting Started with Mintlify

## Prerequisites

Before you begin, make sure you have:

- Node.js (^16.20 || ^18.18 || >=20.17).


## Install and Use Mintlify Locally

**1.** Install the [CLI](https://www.npmjs.com/package/mint):

```
npm i -g mint
```

**2.** Navigate to the docs directory (where the docs.json file is located) and execute the following command:

```
mint dev
```

Alternatively, if you do not want to install the CLI globally, you can use a run a one-time script:

```
npx mint dev
```

A local preview of your documentation will be available at `http://localhost:3000`.
Please note that the rendering behavior in the local preview may differ from the deployed version. The deployed result should be considered as the final reference.