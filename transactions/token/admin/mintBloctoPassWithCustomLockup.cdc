import BloctoToken from "../../../contracts/flow/token/BloctoToken.cdc"
import NonFungibleToken from "../../../contracts/flow/token/NonFungibleToken.cdc"
import BloctoPass from "../../../contracts/flow/token/BloctoPass.cdc"

transaction(address: Address, amount: UFix64, unlockTime: UFix64) {

    prepare(signer: AuthAccount) {
        let minter = signer
            .borrow<&BloctoPass.NFTMinter>(from: BloctoPass.MinterStoragePath)
            ?? panic("Signer is not the admin")

        let nftCollectionRef = getAccount(address).getCapability(BloctoPass.CollectionPublicPath)
            .borrow<&{NonFungibleToken.CollectionPublic, BloctoPass.CollectionPublic}>()
            ?? panic("Could not borrow blocto pass collection public reference")

        let bltVaultRef = signer
            .borrow<&BloctoToken.Vault>(from: BloctoToken.TokenStoragePath)
            ?? panic("Cannot get BLT vault reference")
        
        let bltVault <- bltVaultRef.withdraw(amount: amount)

        let metadata: {String: String} = {
            "origin": "Private Sale"
        }

        let lockupSchedule: {UFix64: UFix64} = {
            0.0                : 1.0,
            unlockTime - 300.0 : 1.0,
            unlockTime - 240.0 : 0.8,
            unlockTime - 180.0 : 0.6,
            unlockTime - 120.0 : 0.4,
            unlockTime - 60.0  : 0.2,
            unlockTime         : 0.0
        }

        minter.mintNFTWithCustomLockup(
            recipient: nftCollectionRef,
            metadata: metadata,
            vault: <- bltVault,
            lockupSchedule: lockupSchedule
        )
    }
}
