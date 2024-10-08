const fs = require('fs');

const GENERATE_URLS_LAMBDA = "https://7462iuvrevrwndflwr5r6nf2340owkmz.lambda-url.ap-southeast-2.on.aws/"
const COMPILE_URLS_LAMBDA = "https://w6myokcnql4lw2oel27xj52njy0cfrto.lambda-url.ap-southeast-2.on.aws/";

type GenerateUrlsRequest = {
    files: string[];
}

type GenerateUrlsResponse = {
    id: string;
    presigned_urls: string[];
};

type CompileRequest = {
    id: string
}

async function main() {
    const request: GenerateUrlsRequest = {
        files: ["index.ts"]
    };

    console.log(JSON.stringify(request))
    const response = await fetch(GENERATE_URLS_LAMBDA, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(request)
        }
    )

    if (!response.ok) {
        console.error("Failed to fetch:", response.status, response.statusText);
        return
    }

    const data: GenerateUrlsResponse = await response.json();
    const filePath = "./index.ts";
    console.log("URLs response:", data);

    try {
        await uploadFileToS3(data.presigned_urls[0], filePath);
    } catch(err: any) {
        console.log(err)
        return;
    }

    try {
        await compile(data.id)
    } catch (err: any) {
        console.log(err)
        return;
    }
}

async function compile(id: string) {
    const request: CompileRequest = {
        id
    }
    const response = await fetch(COMPILE_URLS_LAMBDA, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
    })

    console.log("compile response:", response);

    if (!response.ok) {
        throw new Error("Failed to fetch: " + response.status + " "+ response.statusText);
    }
}

async function uploadFileToS3(presignedUrl: string, filePath: string) {
    const fileBuffer = fs.readFileSync(filePath);

    const uploadResponse = await fetch(presignedUrl, {
        method: 'PUT',
        body: fileBuffer,
        headers: {
            'Content-Type': 'application/octet-stream'
        }
    });

    if (!uploadResponse.ok) {
        throw new Error(`Failed to upload file: ${uploadResponse.statusText}`);
    }
}

main().then(()=> {console.log("done")})