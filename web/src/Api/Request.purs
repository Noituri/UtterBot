module Utter.Api.Request (BaseURL(..), exchangeCode) where

import Prelude

import Affjax (Request, printError, request)
import Affjax.RequestBody as RB
import Affjax.RequestHeader (RequestHeader(..))
import Affjax.ResponseFormat as RF
import Data.Argonaut.Core (Json)
import Data.Argonaut.Decode.Struct.Tolerant as Tolerant
import Data.Argonaut.Encode (encodeJson)
import Data.Bifunctor (bimap)
import Data.Either (Either(..))
import Data.HTTP.Method (Method(..))
import Data.Maybe (Maybe(..))
import Data.Tuple (Tuple(..))
import Effect.Aff.Class (class MonadAff, liftAff)
import Routing.Duplex (print)
import Utter.Api.Endpoint (Endpoint(..), endpointUrl)
import Utter.Capability.Logger (class Logger, log)
import Utter.Data.User (User)

newtype BaseURL = BaseURL String

data RequestMethod
  = Get
  | Post (Maybe Json)

type RequestOptions =
  { endpoint :: Endpoint
  , method :: RequestMethod
  }

defaultRequest :: RequestOptions -> Request Json
defaultRequest { endpoint, method } =
  { method: Left method
  , url: endpointUrl endpoint
  , headers: []
  , content: RB.json <$> body
  , username: Nothing
  , password: Nothing
  , withCredentials: false
  , responseFormat: RF.json
  }
  where
  Tuple method body = case method of
    Get -> Tuple GET Nothing
    Post b -> Tuple POST b

exchangeCode :: forall m. Logger m => MonadAff m => String -> m (Either String User)
exchangeCode code = do
  log $ endpointUrl ExchangeCode
  res <- liftAff $ request $ defaultRequest { endpoint: ExchangeCode, method: Post $ Just $ encodeJson { code } }
  pure $ decodeExchangeCode =<< bimap printError _.body res

decodeExchangeCode :: Json -> Either String User
decodeExchangeCode = Tolerant.decodeJson