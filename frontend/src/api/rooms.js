export async function CreateRoom(name) {
    const response = await fetch("/api/v1/room", {
        method: "POST",
        mode: "cors",
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            name: name
        })
    });

    return response.json();
}

export async function JoinRoom(key) {
    const response = await fetch(`/api/v1/room/${key}`);

    return response.json();
}