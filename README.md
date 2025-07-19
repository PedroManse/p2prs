# Peer to Peer file transfer

<style>
a.anchor {
    color: black;
    text-decoration: underline;
}
</style>

A server is used to register what peers exist and what files they hold.

A client is able to register it self into the server. And with that say what
files are avaliable and what files it wants.


# Client

## Outgoing Actions

1. <a href="#CO-Connect" class="anchor" name="CO-Connect">Connect</a>:
    * Send a list of avaliable files to the server
    * Store other peers
2. <a href="#CO-UpdateFiles" class="anchor" name="CO-UpdateFiles">UpdateFiles</a>:
    * Send the new list of files
3. <a href="#CO-Disconnect" class="anchor" name="CO-Disconnect">Disconnect</a>:
    * Send the Disconnect action to the server
4. <a href="#CO-RequestFile" class="anchor" name="CO-RequestFile">RequestFile</a>:
    * Send the request of a file directly to a peer, with it's path

## Incoming Actions

1. <a href="#CI-RegisterPeer" class="anchor" name="CI-RegisterPeer">RegisterPeer</a>:
    * Create from [RequestFile](#SO-RegisterPeer)
    * Store the new peer and it's file list
2. <a href="#CI-UpdatePeer" class="anchor" name="CI-UpdatePeer">UpdatePeer</a>:
    * Create from [RequestFile](#SO-UpdatePeer)
    * Update the peer's file list
3. <a href="#CI-UnregisterPeer" class="anchor" name="CI-UnregisterPeer">UnregisterPeer</a>:
    * Create from [RequestFile](#SO-UnregisterPeer)
    * Remove the peer
4. <a href="#CI-RequestFile" class="anchor" name="CI-RequestFile">RequestFile</a>:
    * Create from [RequestFile](#CO-RequestFile)
    * Send the file requested to another peer

# Server

## Incoming Actions

1. <a href="#SI-Connect" class="anchor" name="SI-Connect">Connect</a>:
    * Create from [Connect](#CO-Connect)
    * Associate the client's IP with their file list
    * Propagate the client's creation with [RegisterPeer](#SO-RegisterPeer)
    * Tell the new client about old clients
2. <a href="#SI-UpdateFiles" class="anchor" name="SI-UpdateFiles">UpdateFiles</a>:
    * Create from [UpdateFiles](#CO-UpdateFiles)
    * Update the client's file listing
    * Propagate the client's changes with [UpdatePeer](#SO-UpdatePeer)
3. <a href="#SI-Disconnect" class="anchor" name="SI-Disconnect">Disconnect</a>:
    * Create from [Disconnect](#CO-Disconnect)
    * Unregister a peer with [UnregisterPeer](#SO-UnregisterPeer)

## Outgoing Actions

1. <a href="#SO-RegisterPeer" class="anchor" name="SO-RegisterPeer">RegisterPeer</a>:
    * Propagate the client's IP and file list
2. <a href="#SO-UpdatePeer" class="anchor" name="SO-UpdatePeer">UpdatePeer</a>:
    * Update a client's file listing
3. <a href="#SO-UnregisterPeer" class="anchor" name="SO-UnregisterPeer">UnregisterPeer</a>:
    * Propagate the client's disconnection
