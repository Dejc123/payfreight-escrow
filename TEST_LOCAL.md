# PayFreight.io - Local Test Guide

<!--
Running Locally
Open your terminal and navigate to the backend directory:

Bash

cd backend
Install dependencies (run this only the first time):

Bash

npm init -y
npm install express axios dotenv cors
Start the local server:

Bash

node server.js
Verify success:
You should see the following message in your terminal: PayFreight backend is running on http://localhost:3000
-->

## Prerequisites
- Node.js installed on your machine.
- A `.env` file inside the `backend` folder containing your Circle API key:
  ```env
  CIRCLE_API_KEY=your_circle_api_key_here
