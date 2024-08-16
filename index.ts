const GENERATE_URLS_LAMBDA = "https://7462iuvrevrwndflwr5r6nf2340owkmz.lambda-url.ap-southeast-2.on.aws/"
type GenerateUrlsRequest = {
    files: string[];
}

async function main() {
    const request: GenerateUrlsRequest = {
        files: ["value3"]
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

    if (response.ok) {
        const data = await response.json();
        console.log("URLs response:", data);
    } else {
        console.error("Failed to fetch:", response.status, response.statusText);
    }
}


main().then(()=> {console.log("done")})