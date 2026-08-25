let client = reqwest::new();
let provider = HttpProvider::from_url().erased();

let avatars = AvatarApi::new(&provider, &client)
    .with_swam(SwarmGateway::http())
    .with_arweave()
    .build();

let avatars = AvatarApi::new(
    {
        ethereum: {
            1: &provider,
            ...
        }
        http: &client,
        ipfs: IPFSGateway::http(),
        // IPFSGateway::node() - ipfs-lightclient
        // SwarmGateway::http(),
        // Arweave::http(),
    },
);

///
/// 
let avatar_record = provider.get_avatar("luc.eth").await.unwrap();
let avarar_buffer = avatars.parse(avatar_record).await.unwrap().to_buffer();
